//==============================================================================
//
// Title:		OPC UA Client functions wrapper
// Purpose:		Connect to the server, read/write variables. etc
//
// Created on:	08-MAR-2025 by AD.
// License: MPL-2.0
//
// 21-MAR-2025 - load client from config + GetNodeInfo
//==============================================================================
#![allow(unused_must_use)] //on cleanup unused result #ToDo-fix it
use crate::errors::*;

use opcua::types::StatusCode;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
//use log::warn;
use libc::c_char;
use opcua::{
	client::{Client, ClientBuilder, ClientConfig, IdentityToken, Session, SessionEventLoop},
	core::config::Config,
	crypto::SecurityPolicy,
	types::{
		AttributeId, MessageSecurityMode, NodeId, ReadValueId, TimestampsToReturn, UserTokenPolicy,
		Variant,
	},
};
use std::{
	fmt::Write,
	path::PathBuf,
	sync::Arc,
	{ffi::CString, os::raw::c_int},
};

#[macro_use]
pub mod runtime {
	#[macro_export]
	macro_rules! check_runtime {
		($rt_ptr:expr) => {
			if $rt_ptr.is_null() {
				return ERR_INVALID_RUNTIME;
			}
		};
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn lvClientBuilder(client_out: *mut *mut Client) -> i32 {
	if client_out.is_null() {
		return ERR_INVALID_CLIENT_REF; // Error: null output pointer
	}

	// Make the client configuration
	let client = ClientBuilder::new()
		.application_name("Simple Client")
		.application_uri("urn:SimpleClient")
		.product_uri("urn:SimpleClient")
		.trust_server_certs(true)
		.create_sample_keypair(true)
		.session_retry_limit(3)
		.client()
		.unwrap();

	unsafe {
		// Store the boxed client in the output pointer
		*client_out = Box::into_raw(Box::new(client));
	}

	0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn lvClientBuilderFile(
	config_path_str: *const c_char,
	client_out: *mut *mut Client,
) -> i32 {
	if client_out.is_null() {
		return ERR_INVALID_CLIENT_REF; // Error: null output pointer
	}

	// Make the client configuration
	//let config_file = "";
	let config_path_str = cstr_to_string!(config_path_str);
	//let client = Client::new(ClientConfig::load(&PathBuf::from(config_file)).unwrap());
	let client = Client::new(ClientConfig::load(&PathBuf::from(config_path_str)).unwrap());

	unsafe {
		// Store the boxed client in the output pointer
		*client_out = Box::into_raw(Box::new(client));
	}

	NO_ERR
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_connect_loop(
	rt_ptr: *mut Runtime,
	lv_client: *mut Client,
	url: *const i8,
	session_out: *mut *mut Arc<Session>,
	event_loop_out: *mut *mut Arc<SessionEventLoop>,
) -> i32 {
	if lv_client.is_null() || url.is_null() || session_out.is_null() || event_loop_out.is_null() {
		return -1;
	}
	if rt_ptr.is_null() {
		return ERR_INVALID_RUNTIME;
	}

	// Convert C string to Rust string
	let url_str = unsafe {
		match std::ffi::CStr::from_ptr(url as *const i8).to_str() {
			Ok(s) => s.to_string(),
			Err(_) => return -3,
		}
	};

	// Get the client from the pointer (without dropping it)
	let client = unsafe { &mut *lv_client };

	// Execute the async connection logic
	unsafe {
		let rt = &mut *rt_ptr;
		rt.block_on(async {
			match client
				.connect_to_matching_endpoint(
					(
						url_str.as_ref(),
						SecurityPolicy::None.to_str(),
						MessageSecurityMode::None,
						UserTokenPolicy::anonymous(),
					),
					IdentityToken::Anonymous,
				)
				.await
			{
				Ok((session, event_loop)) => {
					// Store the Arc<Session> directly (it's already an Arc)
					*session_out = Box::into_raw(Box::new(session));
					// Wrap the EventLoop in an Arc before storing
					*event_loop_out = Box::into_raw(Box::new(Arc::new(event_loop)));
					0
				}
				Err(_) => -4,
			}
		})
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_connect_simple(
	rt_ptr: *mut Runtime,
	lv_client: *mut Client,
	url: *const i8,
	session_out: *mut *mut Arc<Session>,
	event_loop_out: *mut *mut Arc<SessionEventLoop>,
	handle_out: *mut *mut JoinHandle<StatusCode>,
) -> i32 {
	check_runtime!(rt_ptr);

	if lv_client.is_null() || url.is_null() || session_out.is_null() || event_loop_out.is_null() {
		return ERR_INVALID_CLIENT_REF;
	}

	// Convert C string to Rust string
	let url_str = unsafe {
		match std::ffi::CStr::from_ptr(url as *const i8).to_str() {
			Ok(s) => s.to_string(),
			Err(_) => return -3,
		}
	};

	// Get the client from the pointer (without dropping it)
	let client = unsafe { &mut *lv_client };

	// Execute the async connection logic
	unsafe {
		let rt = &mut *rt_ptr;
		rt.block_on(async {
			match client
				.connect_to_matching_endpoint(
					(
						url_str.as_ref(),
						SecurityPolicy::None.to_str(),
						MessageSecurityMode::None,
						UserTokenPolicy::anonymous(),
					),
					IdentityToken::Anonymous,
				)
				.await
			{
				Ok((session, event_loop)) => {
					let handle = event_loop.spawn(); //Important!
					session.wait_for_connection().await;

					// Store the Arc<Session> directly (it's already an Arc)
					let session_c = session.clone();
					*session_out = Box::into_raw(Box::new(session));
					*handle_out = Box::into_raw(Box::new(handle));

					let r_v1 = session_c
						.read(
							&[NodeId::new(2, "v1").into()],
							TimestampsToReturn::Both,
							0.0,
						)
						.await;

					match r_v1 {
						Ok(read_values) => {
							if let Some(data_value) = read_values.first() {
								if let Some(variant) = &data_value.value {
									if let Variant::Int32(i32_value) = variant {
										return *i32_value; // Successfully extracted i32 OK IT WORKS!
									} else {
										return -4; // Error code for variant not being an i32
									}
								} else {
									return -5; // Error code for no value in DataValue
								}
							} else {
								return -6; // Error code for no values returned
							}
						}
						Err(_) => return -7, // Error code for read failure
					}
				}
				Err(_) => -8,
			}
		})
	}
}

// GetNode Atributes to LV String

#[allow(unused)]
pub fn read_value_id(attribute: AttributeId, id: impl Into<NodeId>) -> ReadValueId {
	let node_id = id.into();
	ReadValueId {
		node_id,
		attribute_id: attribute as u32,
		..Default::default()
	}
}

#[allow(unused)]
pub fn read_value_ids(attributes: &[AttributeId], id: impl Into<NodeId>) -> Vec<ReadValueId> {
	let node_id = id.into();
	attributes
		.iter()
		.map(|a| read_value_id(*a, &node_id))
		.collect()
}
// Will be better to move common and LabVIEW-specific stuff into labview.rs (may be later)
#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct LStr {
	cnt: i32,
	str: [u8; 0],
}
#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
pub struct LStr {
	cnt: i32,
	str: [u8; 0],
}

type LStrHandle = *mut *mut LStr;

unsafe extern "C" {
	// in latest Rust must be unsafe!
	#[link_name = "NumericArrayResize"]
	// Use link_name if the function is named differently in the DLL
	fn string_resize(
		numeric_type: u32, // This should be u32, based on LabVIEW documentation
		num_dimensions: i32,
		data_handle: *mut LStrHandle, // LabVIEW uses UHandle for array resizing.
		new_size: usize,              // New size of the array
	) -> c_int;
}

unsafe extern "C" {
	#[link_name = "MoveBlock"]
	fn MoveBlockChar(src: *const i8, destination: *mut u8, size: usize);
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_get_node_info(
	rt_ptr: *mut Runtime,
	session_in: *mut Arc<Session>,
	id_u32: u32,
	id_str: *const i8,
	ns: u16,
	id_type: u32,
	mut lv_str: LStrHandle,
) -> i32 {
	check_runtime!(rt_ptr);

	unsafe {
		let rt = &mut *rt_ptr;
		if !session_in.is_null() {
			// let session = Box::from_raw(session_in); //Very bad idea, crashed after few calls!
			let session = &mut *session_in;
			// let id: NodeId = NodeId::new(2, "MyVariable").into(); //Jst for test
			let id: NodeId;
			match id_type {
				1 => id = NodeId::new(0, id_u32).into(), //so works so far
				2 => id = NodeId::new(ns, cstr_to_string!(id_str)).into(),
				_ => return ERR_INVALID_TYPE,
			}

			let r = rt.block_on(async {
				session
					.read(
						&read_value_ids(
							&[
								AttributeId::Value,
								AttributeId::DisplayName,
								AttributeId::BrowseName,
								//AttributeId::NodeClass, //lot of Attributes available
								//AttributeId::NodeId,
								//AttributeId::Historizing,
								//AttributeId::ArrayDimensions,
								//AttributeId::Description,
								//AttributeId::ValueRank,
								//AttributeId::DataType,
								//AttributeId::AccessLevel,
								//AttributeId::UserAccessLevel,
							],
							&id,
						),
						TimestampsToReturn::Both,
						0.0,
					)
					.await
					.unwrap()
			});

			let mut i = 0;
			let mut output = String::new();

			while i < r.len() {
				write!(&mut output, "Attribute {}: {:?}\n", i, r[i])
					.expect("Failed to get attribute");
				i = i + 1;
			}
			let len = output.len();
			string_resize(1, 1, &mut lv_str as *mut LStrHandle, len);

			let c_headers = match CString::new(output) {
				Ok(cs) => cs,
				Err(_) => return -1, // failed to convert to C string
			};
			MoveBlockChar(c_headers.as_ptr(), (**lv_str).str.as_mut_ptr(), len);
			(**lv_str).cnt = len as i32;
		}
	}
	return 0;
}

// Update cleanup function to handle Arc types
#[unsafe(no_mangle)]
pub extern "C" fn lv_cleanup_session(
	rt_ptr: *mut Runtime,
	session_in: *mut Arc<Session>,
	event_loop_in: *mut Arc<SessionEventLoop>,
	handle_in: *mut JoinHandle<StatusCode>,
) -> i32 {
	check_runtime!(rt_ptr);

	unsafe {
		let rt = &mut *rt_ptr;
		if !session_in.is_null() {
			let session = Box::from_raw(session_in);
			let handle = Box::from_raw(handle_in);
			//let session = &mut *session_in; //let try this way, no was better
			//let handle = &mut *handle_in;
			//session.disconnect().await;
			//let result = runtime.block_on(async {
			rt.block_on(async { session.disconnect().await });
			rt.block_on(async { handle.await.unwrap() });
		}
		if !event_loop_in.is_null() {
			let _ = Box::from_raw(event_loop_in);
		}
		//rt.shutdown_background();
	}

	return 0;
}
/*

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct LStr {
	cnt: i32,
	str: [u8; 0],
}

#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
pub struct LStr {
	cnt: i32,
	str: [u8; 0],
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct LStr1Darray {
	dim_size: i32,
	node_ru: [*mut *mut LStr; 9999],
}

#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
pub struct LStr1Darray {
	dim_size: i32,
	node_ru: [*mut *mut LStr; 9999],
}

//==============================================================================
//
// #ToDo: Subscription will be the next iteration
// check https://forums.ni.com/t5/LabVIEW/How-to-pass-and-set-Variants-in-the-DLL/m-p/4428062#M1305803

use std::ptr::addr_of;
#[unsafe(no_mangle)]
pub extern "C" fn lv_subscribe_to_variables_i32var(
	rt_ptr: *mut Runtime,
	lv_session: *mut Arc<Session>,
	ns: u16,
	user_event_ref: *mut *mut c_void,
	data: *mut c_void,
	node_path_array: &LStr1DarrayHdl,
	subscription_out: *mut u32,
) -> i32 {
	let session = unsafe { &mut *lv_session };

	// Wrap both raw pointers in thread-safe containers
	let safe_refus = user_event_ref as usize;
	let safe_dataus = data as usize;

	if rt_ptr.is_null() {
		return ERR_INVALID_RUNTIME;
	}

	unsafe {
		let rt = &mut *rt_ptr;

		let subscription_id_res = rt.block_on(async {
			session
				.create_subscription(
					Duration::from_secs(1),
					10,
					30,
					0,
					0,
					true,
					DataChangeCallback::new(move |dv, item| {
						let user_event_ptr = safe_refus as *mut *mut c_void;
						let data_ptr = safe_dataus as *mut c_void;
						// let val = dv.value.as_i32(); //that doesn't work
						//output_debug_string("--callback--");
						let val = if let Some(variant) = &dv.value {
							if let Variant::Int32(i32_value) = variant {
								// *i32_value; // Successfully extracted i32
								let i32_ptr = i32_value as *const i32 as *mut c_void;
								//output_debug_string("callback as i32");
								PostLVUserEvent(*user_event_ptr, i32_ptr)
							} else {
								//output_debug_string("variant not being an i32");
								-4 // Error code for variant not being an i32
							}
						} else {
							-5 // Error code for no value in DataValue
						};
					}),
				)
				.await
		});

		let subscription = {
			match subscription_id_res {
				Ok(subscription_id) => {
					// Create some monitored items

					let mut items_to_create_list = Vec::new();

					//let td1 = (*(*node_path_array)).node_ru.as_ptr();
					//let td1 = std::ptr::addr_of!((*node_path_array).node_ru); // Get raw pointer directly
					//let td1 = std::ptr::addr_of!((*(*node_path_array)).node_ru);

					//let td1 = std::ptr::addr_of!((*(*node_path_array)).node_ru); // Get raw pointer directly
					let td1 = unsafe {
						std::ptr::read_unaligned(addr_of!((*(*node_path_array)).node_ru))
					};

					let dim_size = (*(*node_path_array)).dim_size;

					for i in 0..dim_size {
						//let lstr_ptr = *td1.add(i as usize);
						let lstr_ptr = td1;
						if lstr_ptr.is_null() {
							break;
						}

						let cnt: usize = (*(*lstr_ptr)).cnt as usize;

						let str_ptr: *const u8 = (*(*lstr_ptr)).str.as_ptr();

						// Create a slice from the raw pointer and length
						let slice = slice::from_raw_parts(str_ptr, cnt);
						let name_str: &str = str::from_utf8(slice).unwrap();
						items_to_create_list.push(name_str);
					}

					let items_to_create: Vec<MonitoredItemCreateRequest> = items_to_create_list // ! v1 hard coded !
						.iter()
						.map(|v| NodeId::new(ns, *v).into())
						.collect();

					let _ = rt.block_on(async {
						session
							.create_monitored_items(
								subscription_id,
								TimestampsToReturn::Both,
								items_to_create,
							)
							.await
					});

					*subscription_out = subscription_id;
				}
				Err(_) => return -7, // Error code for read failure
			}
		};
	}
	return 0;
}
*/

#[unsafe(no_mangle)]
pub extern "C" fn lv_delete_subscription(
	rt_ptr: *mut Runtime,
	lv_session: *mut Arc<Session>,
	sub_id: u32,
) -> i32 {
	let session = unsafe { &mut *lv_session };

	if rt_ptr.is_null() {
		return -2;
	}

	unsafe {
		let rt = &mut *rt_ptr;
		rt.block_on(async {
			session.delete_subscription(sub_id).await.unwrap();
		});
	}
	return 0;
}
