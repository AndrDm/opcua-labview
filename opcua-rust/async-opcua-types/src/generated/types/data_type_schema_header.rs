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
pub struct DataTypeSchemaHeader {
    pub namespaces: Option<Vec<opcua::types::string::UAString>>,
    pub structure_data_types: Option<Vec<super::structure_description::StructureDescription>>,
    pub enum_data_types: Option<Vec<super::enum_description::EnumDescription>>,
    pub simple_data_types: Option<Vec<super::simple_type_description::SimpleTypeDescription>>,
}
impl opcua::types::MessageInfo for DataTypeSchemaHeader {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::DataTypeSchemaHeader_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::DataTypeSchemaHeader_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::DataTypeSchemaHeader_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::DataTypeSchemaHeader
    }
}
