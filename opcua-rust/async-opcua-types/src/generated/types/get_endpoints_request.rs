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
pub struct GetEndpointsRequest {
    pub request_header: opcua::types::request_header::RequestHeader,
    pub endpoint_url: opcua::types::string::UAString,
    pub locale_ids: Option<Vec<opcua::types::string::UAString>>,
    pub profile_uris: Option<Vec<opcua::types::string::UAString>>,
}
impl opcua::types::MessageInfo for GetEndpointsRequest {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::GetEndpointsRequest_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::GetEndpointsRequest_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::GetEndpointsRequest_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::GetEndpointsRequest
    }
}
