// This file was autogenerated from schemas/1.05/Opc.Ua.Types.bsd by async-opcua-codegen
//
// DO NOT EDIT THIS FILE

// OPCUA for Rust
// SPDX-License-Identifier: MPL-2.0
// Copyright (C) 2017-2024 Adam Lock, Einar Omang
#[allow(unused)]
mod opcua {
    pub use crate as types;
}
#[opcua::types::ua_encodable]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrustListDataType {
    pub specified_lists: u32,
    pub trusted_certificates: Option<Vec<opcua::types::byte_string::ByteString>>,
    pub trusted_crls: Option<Vec<opcua::types::byte_string::ByteString>>,
    pub issuer_certificates: Option<Vec<opcua::types::byte_string::ByteString>>,
    pub issuer_crls: Option<Vec<opcua::types::byte_string::ByteString>>,
}
impl opcua::types::MessageInfo for TrustListDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::TrustListDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::TrustListDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::TrustListDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::TrustListDataType
    }
}
