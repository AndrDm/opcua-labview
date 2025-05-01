#![allow(static_mut_refs)] // because of SERVER_GLOBAL_RUNTIME
#![allow(unused_variables)] //#ToDo: rt is unused (because globa used)
#![allow(unused_must_use)] //#ToDo: check result in lv_start_server(...)
//==============================================================================
//
// Title:		Server functions wrapper
// Purpose:		Start/Stop Server and serve variables.
//
// Created on:	14-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================

use crate::errors::*;

use std::{
	sync::{Arc, Mutex},
	thread,
};

use tokio::{
	runtime::{Builder, Runtime},
	sync::oneshot,
};

use libc::c_char;
use opcua::{
	server::{
		node_manager::memory::{
			InMemoryNodeManager, /* NamespaceMetadata, */ SimpleNodeManager,
			SimpleNodeManagerImpl, simple_node_manager,
		},
		{Server, ServerBuilder, ServerHandle},
	},
	types::{BuildInfo, DateTime, NodeId},
};

use opcua::server::diagnostics::node_manager::NamespaceMetadata;

pub static mut SERVER_GLOBAL_RUNTIME: Option<Arc<Mutex<Runtime>>> = None;

#[unsafe(no_mangle)]
pub extern "C" fn lv_new_server_runtime() -> *mut Runtime {
	let runtime = Builder::new_current_thread().enable_all().build().unwrap();
	unsafe {
		SERVER_GLOBAL_RUNTIME = Some(Arc::new(Mutex::new(runtime)));
	}

	Box::into_raw(Box::new(Runtime::new().unwrap()))
}

#[unsafe(no_mangle)]
pub extern "C" fn lvServerBuilder(
	config_path_str: *const c_char,
	rt_ptr: *mut Runtime,
	server_out: *mut *mut Server,
	handle_out: *mut *mut ServerHandle,
	manager_out: *mut *mut Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
) -> i32 {
	check_null!(server_out, ERR_NULL_POINTER);
	check_null!(handle_out, ERR_NULL_POINTER);
	check_null!(manager_out, ERR_NULL_POINTER);

	let config_path_str = cstr_to_string!(config_path_str);
	// Execute the async connection logic
	unsafe {
		let rt1 = &mut *rt_ptr;

		let rt = unsafe { SERVER_GLOBAL_RUNTIME.as_ref().unwrap() };

		rt.lock().unwrap().block_on(async move {
			let (server, handle, manager) = ss(config_path_str).await;
			*server_out = Box::into_raw(Box::new(server));
			*handle_out = Box::into_raw(Box::new(handle));
			*manager_out = Box::into_raw(Box::new(manager));
		});
	}

	0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_stop_server(
	rt_ptr: *mut Runtime,
	handle_in: *mut ServerHandle,
	join_handle_in: *mut Arc<std::thread::JoinHandle<()>>,
) -> i32 {
	check_null!(handle_in, ERR_INVALID_SERVER_REF);
	check_null!(rt_ptr, ERR_INVALID_RUNTIME);
	check_null!(join_handle_in, ERR_INVALID_SERVER_REF);

	unsafe {
		let rt1 = &mut *rt_ptr;

		let rt = unsafe { SERVER_GLOBAL_RUNTIME.as_ref().unwrap() };

		let handle = &mut *handle_in;
		//let join_handle = &mut *join_handle_in;

		handle.cancel(); //as in provided example

		let rt_handle = rt.lock().unwrap().handle().clone();
		rt_handle.block_on(async move {
			//	r.await;
			//rt.shutdown_background();
		});
	}

	return 0;
}

async fn ss(
	config_path_str: String,
) -> (
	Server,
	ServerHandle,
	Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
) {
	let (server, handle) = ServerBuilder::new()
		.with_config_from(config_path_str)
		.build_info(BuildInfo {
			product_uri: "https://github.com/freeopcua/async-opcua".into(),
			manufacturer_name: "Rust OPC-UA".into(),
			product_name: "Rust OPC-UA sample server".into(),
			software_version: "0.1.0".into(),
			build_number: "1".into(),
			build_date: DateTime::now(),
		})
		.with_node_manager(simple_node_manager(
			NamespaceMetadata {
				namespace_uri: "urn:SimpleServer".to_owned(),
				..Default::default()
			},
			"simple",
		))
		.trust_client_certs(true)
		.build()
		.unwrap();
	let node_manager = handle
		.node_managers()
		.get_of_type::<SimpleNodeManager>()
		.unwrap();

	let ns = handle.get_namespace_index("urn:SimpleServer").unwrap();

	(server, handle, node_manager)
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_start_server(
	rt_ptr: *mut Runtime,
	lv_server: *mut Server,
	server_handle_out: *mut *mut (), //not needed in general
	join_handle_out: *mut *mut Arc<std::thread::JoinHandle<()>>,
) -> i32 {
	// Create a Tokio runtime
	// let rt = Runtime::new()?;
	if rt_ptr.is_null() {
		return ERR_INVALID_RUNTIME;
	}

	// Execute the async connection logic
	unsafe {
		let rt1 = &mut *rt_ptr;

		let rt = unsafe { SERVER_GLOBAL_RUNTIME.as_ref().unwrap() };
		let server = &mut *lv_server;

		rt.lock().unwrap().block_on(async {
			//server.run().await.unwrap();
			//*server_out = Box::into_raw(Box::new(server));
		});

		// Create a channel to send a signal to the server thread to start
		let (tx, rx) = oneshot::channel();

		// Start the server in a separate thread
		let server_handle = {
			//let rt = rt.clone();
			let handle = Arc::new(thread::spawn(move || {
				// Clone the runtime to use in the thread
				//let rt = rt.clone();
				rt.lock().unwrap().block_on(async {
					// Wait for the signal to start the server
					rx.await.unwrap();
					server.run().await.unwrap();
					// server running
				});
			}));
			*join_handle_out = Box::into_raw(Box::new(handle));
		};

		// Send the signal to start the server
		tx.send(());

		// Return the join handle to keep the thread running
		//Ok(server_handle)

		*server_handle_out = Box::into_raw(Box::new(server_handle));
		return 0;
	}
}

//==============================================================================
// Check if the server is running
// Returns 1 if the server is running, 0 otherwise
// In general this will check running tokio runtime, instead of server itself
// #ToDo: check if opcua server is really running
//
#[unsafe(no_mangle)]
pub extern "C" fn lv_is_server_running(
	rt_ptr: *mut Runtime,
	join_handle_in: *mut Arc<std::thread::JoinHandle<()>>,
) -> i32 {
	check_null!(join_handle_in, ERR_INVALID_SERVER_REF);
	check_null!(rt_ptr, ERR_INVALID_RUNTIME);

	unsafe {
		let rt1 = &mut *rt_ptr;

		let rt = unsafe { SERVER_GLOBAL_RUNTIME.as_ref().unwrap() };
		let handle = &mut *join_handle_in;
		if !(handle.is_finished()) {
			return 1;
		} else {
			return 0;
		};
	}
}

//==============================================================================
// Add folder to the server
// the manager_ptr coming from lvServerBuilder()
//

#[unsafe(no_mangle)]
pub extern "C" fn lv_add_folder(
	folder_node_str: *const c_char,
	folder_browse_str: *const c_char,
	folder_display_str: *const c_char,
	ns: u16,
	manager_ptr: *mut Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
	folder_id_out: *mut *mut NodeId,
	//address_space_out: *mut *mut RwLockWriteGuard<'_, RawRwLock, AddressSpace>
) -> i32 {
	unsafe {
		check_null!(manager_ptr, ERR_INVALID_SERVER_REF);
		check_null!(folder_id_out, ERR_NULL_POINTER);

		let manager = &mut *manager_ptr;

		let folder_node_str = cstr_to_string!(folder_node_str);
		let folder_browse_str = cstr_to_string!(folder_browse_str);
		let folder_display_str = cstr_to_string!(folder_display_str);
		let address_space = manager.address_space();
		let mut address_space = address_space.write();

		// Create a sample folder under objects folder
		let sample_folder_id = NodeId::new(ns, folder_node_str); //was "folder"
		address_space.add_folder(
			&sample_folder_id,
			folder_browse_str,
			folder_display_str,
			&NodeId::objects_folder_id(),
		);
		//*address_space_out = Box::into_raw(Box::new(address_space)); //no need
		*folder_id_out = Box::into_raw(Box::new(sample_folder_id));
	}
	0
}
