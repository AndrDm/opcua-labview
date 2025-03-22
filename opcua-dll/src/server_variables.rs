//==============================================================================
//
// Title:		Server Variables, create and hold
// Purpose:		Currently the only scalar Bool and U8...F64 supported
//
// Created on:	14-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================
use libc::c_char;
use opcua::{
	server::{
		ServerHandle,
		address_space::VariableBuilder,
		node_manager::memory::{InMemoryNodeManager, SimpleNodeManagerImpl},
	},
	types::{DataTypeId, DataValue, NodeId},
};
use std::sync::Arc;

use crate::errors::*;

#[unsafe(no_mangle)]
pub extern "C" fn lv_add_variable(
	variable_node_str: *const c_char,
	variable_browse_str: *const c_char,
	variable_display_str: *const c_char,
	ns: u16,
	var_type: u16,
	manager_ptr: *mut Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
	folder_id_ptr: *mut NodeId,
) -> i32 {
	unsafe {
		check_null!(manager_ptr, ERR_INVALID_SERVER_REF);
		check_null!(folder_id_ptr, ERR_INVALID_SERVER_REF);

		let manager = &mut *manager_ptr;
		let folder_id = &mut *folder_id_ptr;
		let variable_node_str = cstr_to_string!(variable_node_str);
		let variable_browse_str = cstr_to_string!(variable_browse_str);
		let variable_display_str = cstr_to_string!(variable_display_str);
		let address_space = manager.address_space();
		let mut address_space = address_space.write();
		let variable_node = NodeId::new(ns, variable_node_str);
		//#ToDo: Refactor to get writable, etc and organized_by from LabVIEW
		match var_type {
			1 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Boolean)
				.value(false)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			2 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::SByte)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			3 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Byte)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			4 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Int16)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			5 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::UInt16)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			6 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Int32)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			7 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::UInt32)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			8 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Int64)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			9 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Int64)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			10 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Float)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),
			11 => VariableBuilder::new(&variable_node, variable_browse_str, variable_display_str)
				.data_type(DataTypeId::Double)
				.value(0)
				.writable()
				.organized_by(&*folder_id)
				.insert(&mut *address_space),

			_ => return ERR_INVALID_TYPE,
		};
	}

	0
}

macro_rules! create_lv_write_variable {
	($fn_name:ident, $value_type:ty) => {
		#[unsafe(no_mangle)]
		pub extern "C" fn $fn_name(
			variable_node_str: *const c_char,
			ns: u16,
			value: $value_type,
			manager_ptr: *mut Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
			server_handle_ptr: *mut ServerHandle,
		) -> i32 {
			unsafe {
				let variable_node_str = cstr_to_string!(variable_node_str);
				let variable_node = NodeId::new(ns, variable_node_str);

				check_null!(manager_ptr, ERR_INVALID_SERVER_REF);
				check_null!(server_handle_ptr, ERR_INVALID_SERVER_REF);

				let manager = &mut *manager_ptr;
				let server_handle = &mut *server_handle_ptr;

				let address_space = manager.address_space();
				let subscriptions = server_handle.subscriptions().clone();

				address_space.force_unlock_write();
				let data_value = DataValue::new_now(value);
				manager
					.set_value(&subscriptions, &variable_node, None, data_value)
					.unwrap();
			}
			return 0;
		}
	};
}

// Create functions for different variable types
create_lv_write_variable!(lv_write_variableBoolean, bool); // 1
create_lv_write_variable!(lv_write_variableSByte, i8); // 2
create_lv_write_variable!(lv_write_variableByte, u8); // 3
create_lv_write_variable!(lv_write_variableInt16, i16); //...
create_lv_write_variable!(lv_write_variableUInt16, u16);
create_lv_write_variable!(lv_write_variableInt32, i32);
create_lv_write_variable!(lv_write_variableUInt32, u32);
create_lv_write_variable!(lv_write_variableInt64, i64);
create_lv_write_variable!(lv_write_variableUInt64, u64);
create_lv_write_variable!(lv_write_variableFloat, f32);
create_lv_write_variable!(lv_write_variableDouble, f64); // 11
// too tired to write the rest
