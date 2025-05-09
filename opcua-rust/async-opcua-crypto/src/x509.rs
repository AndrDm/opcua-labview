// OPCUA for Rust
// SPDX-License-Identifier: MPL-2.0
// Copyright (C) 2017-2024 Adam Lock

//! Wrapper for X509 certificates, and related tooling.

use std::{
    self,
    collections::HashSet,
    fmt::{self, Debug, Formatter},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    result::Result,
};

use chrono::{DateTime, Utc};
use tracing::{error, info, trace, warn};
type ChronoUtc = DateTime<Utc>;

use rsa;
use rsa::pkcs1v15;
use rsa::RsaPublicKey;
use x509_cert::{
    self as x509,
    der::asn1::{Ia5String, OctetString},
    ext::pkix::name::GeneralName,
};

use const_oid;
use x509::builder::Error as BuilderError;
use x509::ext::pkix::name as xname;

use opcua_types::{status_code::StatusCode, ApplicationDescription, ByteString, Error};

use super::{
    hostname,
    pkey::{PrivateKey, PublicKey},
    thumbprint::Thumbprint,
};

const DEFAULT_KEYSIZE: u32 = 2048;
const DEFAULT_COUNTRY: &str = "IE";
const DEFAULT_STATE: &str = "Dublin";

#[derive(Debug, Default)]
/// Alternate names for an X509 certificate.
pub struct AlternateNames {
    /// List of alternative names.
    pub names: x509::ext::pkix::SubjectAltName,
}

impl AlternateNames {
    /// Create a new `AlternateNames` struct with no contents.
    pub fn new() -> Self {
        use x509::ext::pkix::SubjectAltName;
        Self {
            names: SubjectAltName(xname::GeneralNames::new()),
        }
    }

    /// Create a new list of alternate names from a list of addresses.
    pub fn new_from_addresses(ads: Vec<String>) -> Self {
        let mut result = Self::new();
        result.add_addresses(&ads);
        result
    }

    /// `true` if no alternate names are added.
    pub fn is_empty(&self) -> bool {
        self.names.0.is_empty()
    }

    /// Number of alternate names added.
    pub fn len(&self) -> usize {
        self.names.0.len()
    }

    /// Add an IPV4 address as alternate name.
    pub fn add_ipv4(&mut self, ad: &std::net::Ipv4Addr) {
        if let Ok(v) = x509::der::asn1::OctetString::new(ad.octets()) {
            self.names.0.push(xname::GeneralName::IpAddress(v));
        }
    }

    /// Add an IPV6 address as alternate name.
    pub fn add_ipv6(&mut self, ad: &std::net::Ipv6Addr) {
        if let Ok(v) = x509::der::asn1::OctetString::new(ad.octets()) {
            self.names.0.push(xname::GeneralName::IpAddress(v))
        }
    }

    /// Add a DNS name as alternate name.
    pub fn add_dns(&mut self, v: impl AsRef<str>) {
        if let Ok(v) = x509::der::asn1::Ia5String::new(v.as_ref()) {
            self.names.0.push(xname::GeneralName::DnsName(v));
        }
    }

    /// Add an IP or hostname.
    pub fn add_address(&mut self, v: impl AsRef<str>) {
        let v = v.as_ref();
        {
            if let Ok(ip) = v.parse::<std::net::Ipv4Addr>() {
                self.add_ipv4(&ip);
                return;
            }
        }
        {
            if let Ok(r) = v.parse::<std::net::Ipv6Addr>() {
                self.add_ipv6(&r);
                return;
            }
        }
        self.add_dns(v);
    }

    /// Add a URI.
    pub fn add_uri(&mut self, v: &str) {
        if let Ok(uri) = Ia5String::new(v) {
            self.names
                .0
                .push(xname::GeneralName::UniformResourceIdentifier(uri));
        }
    }

    /// Add a list of addresses.
    pub fn add_addresses(&mut self, ads: &[String]) {
        ads.iter().for_each(|h| {
            self.add_address(h);
        })
    }

    fn convert_name(name: &x509::ext::pkix::name::GeneralName) -> Option<String> {
        match name {
            GeneralName::DnsName(val) => Some(val.to_string()),
            GeneralName::DirectoryName(val) => Some(val.to_string()),
            GeneralName::Rfc822Name(val) => Some(val.to_string()),
            GeneralName::UniformResourceIdentifier(val) => Some(val.to_string()),
            GeneralName::IpAddress(val) => {
                let bytes = val.as_bytes();
                match bytes.len() {
                    4 => Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string()),

                    16 => {
                        let a = ((bytes[0] as u16) << 8) | bytes[1] as u16;
                        let b = ((bytes[2] as u16) << 8) | bytes[3] as u16;
                        let c = ((bytes[4] as u16) << 8) | bytes[5] as u16;
                        let d = ((bytes[6] as u16) << 8) | bytes[7] as u16;
                        let e = ((bytes[8] as u16) << 8) | bytes[9] as u16;
                        let f = ((bytes[10] as u16) << 8) | bytes[11] as u16;
                        let g = ((bytes[12] as u16) << 8) | bytes[13] as u16;
                        let h = ((bytes[14] as u16) << 8) | bytes[15] as u16;
                        Some(Ipv6Addr::new(a, b, c, d, e, f, g, h).to_string())
                    }
                    _ => None,
                }
            }

            _ => None,
        }
    }

    /// Iterate over all the registered names.
    pub fn iter(&self) -> impl Iterator<Item = String> + '_ {
        AlternateNamesStringIterator {
            source: &self.names.0,
            index: 0,
        }
    }
}

struct AlternateNamesStringIterator<'a> {
    source: &'a xname::GeneralNames,
    index: usize,
}

impl Iterator for AlternateNamesStringIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.source.len() {
            let converted = AlternateNames::convert_name(&self.source[self.index]);
            self.index += 1;

            match converted {
                None => Some("".to_string()),
                Some(val) => Some(val),
            }
        } else {
            None
        }
    }
}

impl From<Vec<String>> for AlternateNames {
    fn from(source: Vec<String>) -> Self {
        Self::new_from_addresses(source)
    }
}

/// Data for constructing an X509 certificate.
pub struct X509Data {
    /// Requested key size.
    pub key_size: u32,
    /// Certificate CN.
    pub common_name: String,
    /// Certificate organization.
    pub organization: String,
    /// Certificate organizational unit.
    pub organizational_unit: String,
    /// Certificate country.
    pub country: String,
    /// Certificate state.
    pub state: String,
    /// A list of alternate host names as text. The first entry is expected to be the application uri.
    /// The remainder are treated as IP addresses or DNS names depending on whether they parse as IPv4, IPv6 or neither.
    /// IP addresses are expected to be in their canonical form and you will run into trouble
    /// especially in IPv6 if they are not because string comparison may be used during validation.
    /// e.g. IPv6 canonical format shortens addresses by stripping leading zeros, sequences of zeros
    /// and using lowercase hex.
    pub alt_host_names: AlternateNames,
    /// The number of days the certificate is valid for, i.e. it will be valid from now until now + duration_days.
    pub certificate_duration_days: u32,
}

impl From<(ApplicationDescription, Option<Vec<String>>)> for X509Data {
    fn from(v: (ApplicationDescription, Option<Vec<String>>)) -> Self {
        let (application_description, addresses) = v;
        let application_uri = application_description.application_uri.as_ref();
        let mut alt_host_names = AlternateNames::new();
        Self::compute_alt_host_names(
            &mut alt_host_names,
            application_uri,
            addresses,
            true,
            true,
            true,
        );
        X509Data {
            key_size: DEFAULT_KEYSIZE,
            common_name: application_description.application_name.to_string(),
            organization: application_description.application_name.to_string(),
            organizational_unit: application_description.application_name.to_string(),
            country: DEFAULT_COUNTRY.to_string(),
            state: DEFAULT_STATE.to_string(),
            alt_host_names,
            certificate_duration_days: 365,
        }
    }
}

impl From<ApplicationDescription> for X509Data {
    fn from(v: ApplicationDescription) -> Self {
        X509Data::from((v, None))
    }
}

impl X509Data {
    /// Gets a list of possible dns hostnames for this device
    pub fn computer_hostnames() -> Vec<String> {
        let mut result = Vec::with_capacity(2);

        if let Ok(hostname) = hostname() {
            if !hostname.is_empty() {
                result.push(hostname);
            }
        }
        if result.is_empty() {
            // Look for environment vars
            if let Ok(machine_name) = std::env::var("COMPUTERNAME") {
                result.push(machine_name);
            }
            if let Ok(machine_name) = std::env::var("NAME") {
                result.push(machine_name);
            }
        }

        result
    }

    /// Create `AlternateNames` from the current host and application URI, with
    /// an optional extra list of addresses.
    pub fn alt_host_names(
        application_uri: &str,
        addresses: Option<Vec<String>>,
        add_localhost: bool,
        add_computer_name: bool,
        add_ip_addresses: bool,
    ) -> AlternateNames {
        let mut result = AlternateNames::new();
        Self::compute_alt_host_names(
            &mut result,
            application_uri,
            addresses,
            add_localhost,
            add_computer_name,
            add_ip_addresses,
        );
        result
    }

    /// Creates a list of uri + DNS hostnames using the supplied arguments
    fn compute_alt_host_names(
        result: &mut AlternateNames,
        application_uri: &str,
        addresses: Option<Vec<String>>,
        add_localhost: bool,
        add_computer_name: bool,
        add_ip_addresses: bool,
    ) {
        // The first name is the application uri

        result.add_uri(application_uri);

        // Addresses supplied by caller
        if let Some(addresses) = addresses {
            result.add_addresses(&addresses);
        }

        // The remainder are alternative IP/DNS entries
        if add_localhost {
            result.add_address("localhost");
            if add_ip_addresses {
                result.add_address("127.0.0.1");
                result.add_address("::1");
            }
        }
        // Get the machine name / ip address
        if add_computer_name {
            let computer_hostnames = Self::computer_hostnames();
            if add_ip_addresses {
                let mut ipaddresses = HashSet::new();
                // Iterate hostnames, produce a set of ip addresses from lookup, using set to eliminate duplicates
                computer_hostnames.iter().for_each(|h| {
                    ipaddresses.extend(Self::ipaddresses_from_hostname(h));
                });
                result.add_addresses(&computer_hostnames);
                ipaddresses.iter().for_each(|v| {
                    result.add_address(v);
                });
            } else {
                result.add_addresses(&computer_hostnames);
            }
        }
    }

    /// Do a hostname lookup, find matching IP addresses
    fn ipaddresses_from_hostname(hostname: &str) -> Vec<String> {
        // Get ip addresses
        if let Ok(addresses) = (hostname, 0u16).to_socket_addrs() {
            addresses
                .map(|addr| match addr {
                    SocketAddr::V4(addr) => addr.ip().to_string(),
                    SocketAddr::V6(addr) => addr.ip().to_string(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Creates a sample certificate for testing, sample purposes only
    pub fn sample_cert() -> X509Data {
        let mut alt_host_names = AlternateNames::new();
        Self::compute_alt_host_names(&mut alt_host_names, "urn:OPCUADemo", None, true, true, true);
        X509Data {
            key_size: 2048,
            common_name: "OPC UA Demo Key".to_string(),
            organization: "OPC UA for Rust".to_string(),
            organizational_unit: "OPC UA for Rust".to_string(),
            country: DEFAULT_COUNTRY.to_string(),
            state: DEFAULT_STATE.to_string(),
            alt_host_names,
            certificate_duration_days: 365,
        }
    }
}

#[derive(Debug)]
/// Error returned when handling X509 certificates.
pub struct X509Error;

impl fmt::Display for X509Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X509Error")
    }
}

impl std::error::Error for X509Error {}

impl From<x509::der::Error> for X509Error {
    fn from(_err: x509::der::Error) -> Self {
        X509Error
    }
}

#[derive(Clone)]
/// Wrapper around an X509 certificate.
pub struct X509 {
    value: x509::certificate::Certificate,
}

impl Debug for X509 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // This impl will not write out the cert, and exists to keep derive happy
        // on structs that contain an X509 instance
        write!(f, "[x509]")
    }
}

impl X509 {
    /// Load an X509 certificate from a pem file.
    pub fn from_pem(data: &[u8]) -> Result<Self, X509Error> {
        use der::Decode;
        use der::Reader;
        use x509::der;

        let mut reader = der::PemReader::new(data)?;
        let val = x509::certificate::Certificate::decode(&mut reader)?;
        let valf = reader.finish(val)?;
        Ok(X509 { value: valf })

        //keep certificate chain for another story
        //let r = x509::certificate::Certificate::load_pem_chain(data);
    }

    /// Load an X509 certificate from a der file.
    pub fn from_der(data: &[u8]) -> Result<Self, X509Error> {
        use x509::der::Decode;

        let val = x509::certificate::Certificate::from_der(data)?;
        Ok(X509 { value: val })
    }

    /// Serialize the X509 file to a der file.
    pub fn to_der(&self) -> Result<Vec<u8>, X509Error> {
        use x509_cert::der::Encode;
        let data = self.value.to_der()?;
        Ok(data)

        /*
        let length = self.value.encoded_len()?;
        let size : u32 = length.into();

                let mut data: Vec<u8> = vec![0;size as usize];
                let mut slice =  x509::der::SliceWriter::new(&mut data);
                self.value.encode(&mut slice)?;
                Ok(data)
        */
    }

    /// Creates a self-signed X509v3 certificate and public/private key from the supplied creation args.
    /// The certificate identifies an instance of the application running on a host as well
    /// as the public key. The PKey holds the corresponding public/private key. Note that if
    /// the pkey is stored by cert store, then only the private key will be written. The public key
    /// is only ever stored with the cert.
    ///
    /// See Part 6 Table 23 for full set of requirements
    ///
    /// In particular, application instance cert requires subjectAltName to specify alternate
    /// hostnames / ip addresses that the host runs on.
    pub fn cert_and_pkey(x509_data: &X509Data) -> Result<(Self, PrivateKey), String> {
        // Create a key pair

        let pkey = PrivateKey::new(x509_data.key_size)
            .map_err(|e| format!("Failed to generate RSA private key: {e}"))?;

        // Create an X509 cert to hold the public key
        let cert = Self::from_pkey(&pkey, x509_data)?;

        Ok((cert, pkey))
    }

    fn append_to_name(name: &mut String, param: &str, data: &str) {
        if !data.is_empty() {
            if !name.is_empty() {
                name.push(',');
            }
            name.push_str(param);
            name.push('=');
            name.push_str(data);
        }
    }

    /// Create a certificate from a private key and certificate description.
    pub fn from_pkey(pkey: &PrivateKey, x509_data: &X509Data) -> Result<Self, String> {
        let result = Self::create_from_pkey(pkey, x509_data);

        match result {
            Ok(val) => Ok(val),
            Err(e) => match e {
                BuilderError::Asn1(_) => Err("Invalid der".to_string()),
                BuilderError::PublicKey(_) => Err("Invalid public key".to_string()),
                BuilderError::Signature(_) => Err("Invalid signature".to_string()),
                _ => Err("Invalid".to_string()),
            },
        }
    }

    fn create_from_pkey(pkey: &PrivateKey, x509_data: &X509Data) -> Result<Self, BuilderError> {
        use std::time::Duration;
        use x509_cert::builder::{CertificateBuilder, Profile};
        use x509_cert::name::Name;
        use x509_cert::serial_number::SerialNumber;
        use x509_cert::time::Validity;

        let pub_key;
        {
            let r = pkey.public_key_to_info();
            match r {
                Err(e) => return Err(BuilderError::PublicKey(e)),
                Ok(v) => pub_key = v,
            }
        }

        let validity = Validity::from_now(Duration::new(
            86400 * x509_data.certificate_duration_days as u64,
            0,
        ))
        .unwrap();

        let signing_key = pkcs1v15::SigningKey::<sha2::Sha256>::new(pkey.value.clone());

        let serial_number = SerialNumber::from(42u32);

        let subject;

        {
            let mut issuer = String::new();
            Self::append_to_name(&mut issuer, "CN", &x509_data.common_name);
            Self::append_to_name(&mut issuer, "O", &x509_data.organization);
            Self::append_to_name(&mut issuer, "OU", &x509_data.organizational_unit);
            Self::append_to_name(&mut issuer, "C", &x509_data.country);
            Self::append_to_name(&mut issuer, "ST", &x509_data.state);

            use std::str::FromStr;
            subject = Name::from_str(&issuer)?;
        }

        // Issuer and subject shall be the same for self-signed cert
        let profile = Profile::Manual {
            issuer: Some(subject.clone()),
        };

        // Generate a SKI, and set it as the AKI for the certificate according to Part 6, 6.2.2
        // Generation is as suggested in RFC3280, 4.2.1.2. A 160-bit SHA-1 hash of the public key bitstring.
        use sha1::Digest;
        let mut hasher = sha1::Sha1::new();
        hasher.update(
            pub_key
                .subject_public_key
                .as_bytes()
                .expect("Invalid public key"),
        );
        let ski = hasher.finalize();

        let mut builder = CertificateBuilder::new(
            profile,
            serial_number.clone(),
            validity,
            subject.clone(),
            pub_key,
            &signing_key,
        )?;

        builder.add_extension(&x509::ext::pkix::SubjectKeyIdentifier(
            OctetString::new(ski.as_slice()).unwrap(),
        ))?;
        builder.add_extension(&x509::ext::pkix::AuthorityKeyIdentifier {
            authority_cert_issuer: Some(vec![GeneralName::DirectoryName(subject)]),
            key_identifier: Some(OctetString::new(ski.as_slice()).unwrap()),
            authority_cert_serial_number: Some(serial_number),
        })?;
        builder.add_extension(&x509::ext::pkix::BasicConstraints {
            ca: false,
            path_len_constraint: None,
        })?;

        {
            use x509::ext::pkix::KeyUsage;
            use x509::ext::pkix::KeyUsages;

            let key_usage = KeyUsages::DigitalSignature
                | KeyUsages::NonRepudiation
                | KeyUsages::KeyEncipherment
                | KeyUsages::DataEncipherment
                | KeyUsages::KeyCertSign;
            builder.add_extension(&KeyUsage(key_usage))?;
        }

        {
            use x509::ext::pkix::ExtendedKeyUsage;
            let usage = vec![
                const_oid::db::rfc5280::ID_KP_CLIENT_AUTH,
                const_oid::db::rfc5280::ID_KP_SERVER_AUTH,
            ];
            builder.add_extension(&ExtendedKeyUsage(usage))?;
        }

        {
            if !x509_data.alt_host_names.is_empty() {
                builder.add_extension(&x509_data.alt_host_names.names)?;
            }
        }

        use x509_cert::builder::Builder;
        let built = builder.build()?;

        Ok(X509 { value: built })
    }

    /// Load a certificate from a der byte string.
    pub fn from_byte_string(data: &ByteString) -> Result<X509, Error> {
        if data.is_null() {
            Err(Error::new(
                StatusCode::BadCertificateInvalid,
                "Cannot make certificate from null bytestring",
            ))
        } else {
            let r = Self::from_der(data.value.as_ref().unwrap());
            match r {
                Err(e) => Err(Error::new(StatusCode::BadCertificateInvalid, e)),
                Ok(cert) => Ok(cert),
            }
        }
    }

    /// Returns a ByteString representation of the cert which is DER encoded form of X509v3
    pub fn as_byte_string(&self) -> ByteString {
        let der = self.to_der().unwrap();
        ByteString::from(&der)
    }

    /// Try to get the public key from this certificate.
    pub fn public_key(&self) -> Result<PublicKey, Error> {
        use x509_cert::der::referenced::OwnedToRef;

        let r = RsaPublicKey::try_from(
            self.value
                .tbs_certificate
                .subject_public_key_info
                .owned_to_ref(),
        );
        match r {
            Err(e) => Err(Error::new(StatusCode::BadCertificateInvalid, e)),
            Ok(v) => Ok(PublicKey { value: v }),
        }
    }

    /// Returns the key length in bits (if possible)
    pub fn key_length(&self) -> Result<usize, X509Error> {
        use crate::pkey::KeySize;

        let r = self.public_key();
        match r {
            Err(_) => Err(X509Error),
            Ok(v) => Ok(v.bit_length()),
        }
    }

    fn get_subject_entry(&self, nid: const_oid::ObjectIdentifier) -> Result<String, X509Error> {
        for dn in self.value.tbs_certificate.subject.0.iter() {
            for tv in dn.0.iter() {
                if tv.oid == nid {
                    return Ok(tv.to_string());
                }
            }
        }

        Err(X509Error)
    }

    /// Produces a subject name string such as "CN=foo/C=IE"
    pub fn subject_name(&self) -> String {
        let r = self.value.tbs_certificate.subject.to_string();
        r.replace(";", "/")
    }

    /// Gets the common name out of the cert
    pub fn common_name(&self) -> Result<String, X509Error> {
        self.get_subject_entry(const_oid::db::rfc4519::COMMON_NAME)
    }

    /// Tests if the certificate is valid for the supplied time using the not before and not
    /// after values on the cert.
    pub fn is_time_valid(&self, now: &DateTime<Utc>) -> Result<(), StatusCode> {
        // Issuer time
        let not_before = self.not_before();
        if let Ok(not_before) = not_before {
            if now.lt(&not_before) {
                error!("Certificate < before date)");
                return Err(StatusCode::BadCertificateTimeInvalid);
            }
        } else {
            // No before time
            error!("Certificate has no before date");
            return Err(StatusCode::BadCertificateInvalid);
        }

        // Expiration time
        let not_after = self.not_after();
        if let Ok(not_after) = not_after {
            if now.gt(&not_after) {
                error!("Certificate has expired (> after date)");
                return Err(StatusCode::BadCertificateTimeInvalid);
            }
        } else {
            // No after time
            error!("Certificate has no after date");
            return Err(StatusCode::BadCertificateInvalid);
        }

        trace!("Certificate is valid for this time");
        Ok(())
    }

    fn get_alternate_names(&self) -> Option<x509::ext::pkix::name::GeneralNames> {
        use x509::ext::pkix::SubjectAltName;

        let r: Result<Option<(bool, SubjectAltName)>, _> = self.value.tbs_certificate.get();
        match r {
            Err(_) => None,
            Ok(option) => match option {
                None => None,
                Some(v) => {
                    Some(v.1 .0) //the second field of option (ie SubjectAltName) then the first field
                }
            },
        }
    }

    /// Tests if the supplied hostname matches any of the dns alt subject name entries on the cert
    pub fn is_hostname_valid(&self, hostname: &str) -> Result<(), StatusCode> {
        trace!("is_hostname_valid against {} on cert", hostname);
        // Look through alt subject names for a matching entry
        if hostname.is_empty() {
            error!("Hostname is empty");
            Err(StatusCode::BadCertificateHostNameInvalid)
        } else if let Some(subject_alt_names) = self.get_alternate_names() {
            let found = subject_alt_names
                .iter()
                .skip(1) //skip the application uri
                .any(|n| {
                    let name = AlternateNames::convert_name(n);
                    match name {
                        Some(val) => val.eq_ignore_ascii_case(hostname),
                        _ => false,
                    }
                });
            if found {
                info!("Certificate host name {} is good", hostname);
                Ok(())
            } else {
                warn!("Did not find hostname {hostname} in alt names {subject_alt_names:?}");
                Err(StatusCode::BadCertificateHostNameInvalid)
            }
        } else {
            // No alt names
            error!("Cert has no subject alt names at all");
            Err(StatusCode::BadCertificateHostNameInvalid)
        }
    }

    /// Tests if the supplied application uri matches the uri alt subject name entry on the cert
    pub fn is_application_uri_valid(&self, application_uri: &str) -> Result<(), StatusCode> {
        // Expecting the first subject alternative name to be a uri that matches with the supplied
        // application uri
        if let Some(alt_names) = self.get_alternate_names() {
            if !alt_names.is_empty() {
                match AlternateNames::convert_name(&alt_names[0]) {
                    Some(val) => {
                        if val == application_uri {
                            Ok(())
                        } else {
                            error!(
                                "Application uri {} does not match first alt name {}",
                                application_uri, val
                            );
                            Err(StatusCode::BadCertificateUriInvalid)
                        }
                    }

                    _ => {
                        error!("Alternate name {:?} cannot be converted", alt_names[0]);
                        Err(StatusCode::BadCertificateUriInvalid)
                    }
                }
            } else {
                error!("Cert has zero subject alt names");
                Err(StatusCode::BadCertificateUriInvalid)
            }
        } else {
            error!("Cert has no subject alt names at all");
            // No alt names
            Err(StatusCode::BadCertificateUriInvalid)
        }
    }

    /// OPC UA Part 6 MessageChunk structure
    ///
    /// The thumbprint is the SHA1 digest of the DER form of the certificate. The hash is 160 bits
    /// (20 bytes) in length and is sent in some secure conversation headers.
    ///
    /// The thumbprint might be used by the server / client for look-up purposes.
    pub fn thumbprint(&self) -> Thumbprint {
        use sha1::Digest;
        use x509_cert::der::Encode;

        let der = self.value.to_der().unwrap();

        let mut hasher = sha1::Sha1::new();
        hasher.update(&der);
        let digest = hasher.finalize();
        Thumbprint::new(&digest)
    }

    /// Turn the Asn1 values into useful portable types
    pub fn not_before(&self) -> Result<ChronoUtc, X509Error> {
        let dur = self
            .value
            .tbs_certificate
            .validity
            .not_before
            .to_unix_duration();
        let r = ChronoUtc::from_timestamp_micros(dur.as_micros() as i64);
        match r {
            None => Err(X509Error),
            Some(val) => Ok(val),
        }
    }

    /// Turn the Asn1 values into useful portable types
    pub fn not_after(&self) -> Result<ChronoUtc, X509Error> {
        let dur = self
            .value
            .tbs_certificate
            .validity
            .not_after
            .to_unix_duration();
        let r = ChronoUtc::from_timestamp_micros(dur.as_micros() as i64);
        match r {
            None => Err(X509Error),
            Some(val) => Ok(val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
        #[test]
        fn parse_asn1_date_test() {
            use chrono::{Datelike, Timelike};

            assert!(X509::parse_asn1_date("").is_err());
            assert!(X509::parse_asn1_date("Jan 69 00:00:00 1970").is_err());
            assert!(X509::parse_asn1_date("Feb 21 00:00:00 1970").is_ok());
            assert!(X509::parse_asn1_date("Feb 21 00:00:00 1970 GMT").is_ok());

            let dt: DateTime<Utc> = X509::parse_asn1_date("Feb 21 12:45:30 1999 GMT").unwrap();
            assert_eq!(dt.month(), 2);
            assert_eq!(dt.day(), 21);
            assert_eq!(dt.hour(), 12);
            assert_eq!(dt.minute(), 45);
            assert_eq!(dt.second(), 30);
            assert_eq!(dt.year(), 1999);
        }
    */

    /// This test checks that a cert will validate dns or ip entries in the subject alt host names
    #[test]
    fn alt_hostnames() {
        let mut alt_host_names = AlternateNames::new();
        alt_host_names.add_dns("uri:foo"); //the application uri
        alt_host_names.add_address("host2");
        alt_host_names.add_address("www.google.com");
        alt_host_names.add_address("192.168.1.1");
        alt_host_names.add_address("::1");

        // Create a cert with alt hostnames which are both IP and DNS entries
        let args = X509Data {
            key_size: 2048,
            common_name: "x".to_string(),
            organization: "x.org".to_string(),
            organizational_unit: "x.org ops".to_string(),
            country: "EN".to_string(),
            state: "London".to_string(),
            alt_host_names,
            certificate_duration_days: 60,
        };

        let (x509, _pkey) = X509::cert_and_pkey(&args).unwrap();

        assert!(x509.is_hostname_valid("").is_err());
        assert!(x509.is_hostname_valid("uri:foo").is_err()); // The application uri should not be valid
        assert!(x509.is_hostname_valid("192.168.1.0").is_err());
        assert!(x509.is_hostname_valid("www.cnn.com").is_err());
        assert!(x509.is_hostname_valid("host1").is_err());

        args.alt_host_names.iter().skip(1).for_each(|n| {
            assert!(x509.is_hostname_valid(n.as_str()).is_ok());
        })
    }
}
