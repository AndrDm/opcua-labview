#![allow(unused_unsafe)] //for macros
//==============================================================================
//
// Title:		Some useful snippets and structures for LabVIEW
//
// Created on:	10-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================
use std::ffi::c_void;

//Pay attention to alignment in 32-bit environment
#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
pub struct TD1Variant {
	data_type: u16,
	data_value: TVariant,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct TD1Variant {
	data_type: u16,
	data_value: TVariant,
}
/*
#[repr(C)]
pub struct LStr {
	cnt: i32,
	str: [u8; 0],
}

#[repr(C)]
pub struct LStr1Darray {
	dim_size: i32,
	node_ru: [*mut *mut LStr; 9999],
}

type LStr1DarrayHdl = *mut LStr1Darray;
*/

type TVariant = *mut *mut c_void;
pub type MgErr = i32;

pub enum LVDataTypeId {
	LvBoolean = 1,
	LvSByte = 2,
	LvByte = 3,
	LvInt16 = 4,
	LvUInt16 = 5,
	LvInt32 = 6,
	LvUInt32 = 7,
	LvInt64 = 8,
	LvUInt64 = 9,
	LvFloat = 10,
	LvDouble = 11,
} //currently only support these types

unsafe extern "C" {
	//exported from LabVIEW.exe
	pub fn PostLVUserEvent(user_event_ref: *mut c_void, data: *mut c_void) -> MgErr;
	pub fn LvVariantUnFlattenExp(
		variant: TVariant,
		str: *const u8,
		size: i32,
		version: i32,
		context: i32,
	) -> MgErr;
}

#[macro_export]
macro_rules! cstr_to_string {
	($ptr:expr) => {
		unsafe {
			::std::ffi::CStr::from_ptr($ptr)
				.to_string_lossy()
				.into_owned()
		}
	};
}

macro_rules! check_null {
	($ptr:expr, $err:expr) => {
		if $ptr.is_null() {
			return $err; // Early return with specified error
		}
	};
}
