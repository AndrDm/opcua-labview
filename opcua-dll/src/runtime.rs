//==============================================================================
//
// Title:		Runtime support
// Purpose:		Tokio RunTime Utiities. Currently shutdown is not fully OK
//
// Created on:	14-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio::time::sleep;

/*
#[macro_export]
macro_rules! check_runtime {
	($rt_ptr:expr) => {
		if $rt_ptr.is_null() {
			return ERR_INVALID_RUNTIME;
		}
	};
}
	*/

#[macro_use]
pub mod runtime {
	#[macro_export]
	macro_rules! check_runtime2 {
		($rt_ptr:expr) => {
			if $rt_ptr.is_null() {
				return ERR_INVALID_RUNTIME;
			}
		};
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_new_runtime() -> *mut Runtime {
	Box::into_raw(Box::new(Runtime::new().unwrap()))

	/*
		let rt = {
			runtime::Builder::new_multi_thread()
				.enable_io()
				.build()
				.unwrap()
		};

		Box::into_raw(Box::new(rt))
	*/
}

#[unsafe(no_mangle)]
pub extern "C" fn lv_shutdown_runtime(rt_ptr: *mut Runtime) -> i32 {
	if rt_ptr.is_null() {
		return -1;
	}

	unsafe {
		let rt = Box::from_raw(rt_ptr);
		let handle = rt.handle().clone();
		let (s, r) = oneshot::channel();

		rt.spawn(async move {
			sleep(Duration::from_secs(1)).await;
			let _ = s.send(0);
		});

		handle.block_on(async move {
			let _ = r.await;
			rt.shutdown_background();
		});

		// Return the pointer to the caller to handle deallocation
		//std::mem::forget(rt); // Dangerous! Make sure the caller knows to call Box::into_raw
		return 0;
	}
}
