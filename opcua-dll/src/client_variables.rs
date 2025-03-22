//==============================================================================
//
// Title:		Server Variables, create and hold
// Purpose:		Currently the only scalar Bool and U8...F64 supported
//
// Created on:	14-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================

use opcua::{
	client::Session,
	//crypto::SecurityPolicy, //later
	types::{NodeId, TimestampsToReturn, Variant},
};
use std::{os::raw::*, sync::Arc};
use tokio::runtime::Runtime;

macro_rules! create_lv_read_variable {
	($suffix:ident, $rust_type:ty, $c_type:ty, $variant:ident) => {
		#[unsafe(no_mangle)]
		pub unsafe extern "C" fn $suffix(
			// Space between name and suffix
			rt_ptr: *mut Runtime,
			lv_session: *mut Arc<Session>,
			vurl: *const i8,
			output: *mut $c_type,
		) -> i32 {
			if lv_session.is_null() {
				return -1;
			}
			if rt_ptr.is_null() {
				return -2;
			}

			let session = unsafe { &mut *lv_session };

			let vurl_str = unsafe {
				match std::ffi::CStr::from_ptr(vurl).to_str() {
					Ok(s) => s.to_string(),
					Err(_) => return -3,
				}
			};
			unsafe {
				let rt = &mut *rt_ptr;
				let var = rt.block_on(async {
					session
						.read(
							&[NodeId::new(2, vurl_str).into()],
							TimestampsToReturn::Both,
							0.0,
						)
						.await
				});

				match var {
					Ok(read_values) => {
						if let Some(data_value) = read_values.first() {
							if let Some(variant) = &data_value.value {
								if let Variant::$variant(value) = variant {
									*output = *value as $c_type;

									return 0;
								} else {
									-4 //Type mismatch
								}
							} else {
								-5 //No value
							}
						} else {
							-6
						}
					}
					Err(_) => -7, //Bad quality
				}
			}
		}
	};
}

create_lv_read_variable!(lv_read_variableBoolean, bool, c_short, Boolean); // 1
create_lv_read_variable!(lv_read_variableSByte, i8, c_char, SByte); // 2
create_lv_read_variable!(lv_read_variableByte, u8, c_uchar, Byte); // 3
create_lv_read_variable!(lv_read_variableInt16, i16, c_short, Int16); //...
create_lv_read_variable!(lv_read_variableUInt16, u16, c_ushort, UInt16);
create_lv_read_variable!(lv_read_variableInt32, i32, c_int, Int32);
create_lv_read_variable!(lv_read_variableUInt32, u32, c_uint, UInt32);
create_lv_read_variable!(lv_read_variableInt64, i64, c_longlong, Int64);
create_lv_read_variable!(lv_read_variableUInt64, u64, c_ulonglong, UInt64);
create_lv_read_variable!(lv_read_variableFloat, f32, c_float, Float);
create_lv_read_variable!(lv_read_variableDouble, f64, c_double, Double); // 11
