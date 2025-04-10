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
pub struct CreateSessionRequest {
    pub request_header: opcua::types::request_header::RequestHeader,
    pub client_description: super::application_description::ApplicationDescription,
    pub server_uri: opcua::types::string::UAString,
    pub endpoint_url: opcua::types::string::UAString,
    pub session_name: opcua::types::string::UAString,
    pub client_nonce: opcua::types::byte_string::ByteString,
    pub client_certificate: opcua::types::byte_string::ByteString,
    pub requested_session_timeout: f64,
    pub max_response_message_size: u32,
}
impl opcua::types::MessageInfo for CreateSessionRequest {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::CreateSessionRequest_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::CreateSessionRequest_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::CreateSessionRequest_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::CreateSessionRequest
    }
}
