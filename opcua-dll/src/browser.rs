//==============================================================================
//
// Title:		OPC UA Browser functions wrapper
// Purpose:		Get list of items, write to LabVIEW's array of clusters
//
// Created on:	16-MAR-2025 by AD.
// License: MPL-2.0
//
//==============================================================================
use crate::errors::*;
use opcua::{
	client::Session,
	types::{
		BrowseDescription, BrowseDirection, BrowseResultMask, NodeClassMask, NodeId,
		ReferenceTypeId,
	},
};
use std::{
	sync::Arc,
	{ffi::CString, os::raw::c_int},
};
use tokio::runtime::Runtime;

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct Node {
	dim_size: c_int,
	node_attribute: [NodeAttribute; 1000], // Placeholder, adjust size as needed
}
// Type alias for double pointer

#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct NodeAttribute {
	class: c_int,
	display_name: LStrHandle,
	node_uid: LStrHandle,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct LStr {
	cnt: i32,
	str: [u8; 0],
}

#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
pub struct Node {
	dim_size: c_int,
	node_attribute: [NodeAttribute; 1000], // Placeholder, adjust size as needed
}
// Type alias for double pointer
#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
struct NodeAttribute {
	class: c_int,
	display_name: LStrHandle,
	node_uid: LStrHandle,
}
#[cfg(target_arch = "x86")]
#[repr(C, packed(1))]
struct LStr {
	cnt: i32,
	str: [u8; 0],
}

type NodeHdl = *mut *mut Node;
type LStrHandle = *mut *mut LStr;

unsafe extern "C" {
	//#[link_name = "DSSetHandleSize"]
	fn DSSetHandleSize(nodes: NodeHdl, size: usize);
	fn DSNewHandle(size: usize) -> LStrHandle;
	#[link_name = "MoveBlock"]
	fn MoveBlockChar(src: *const i8, destination: *mut u8, size: usize);
}

#[unsafe(no_mangle)]
pub extern "C" fn lvBrowser(
	rt_ptr: *mut Runtime,
	session_in: *mut Arc<Session>,
	id_u32: u32,
	id_str: *const i8,
	ns: u16,
	id_type: u32,
	nodes: NodeHdl,
) -> i32 {
	check_null!(rt_ptr, ERR_NULL_POINTER);
	check_null!(session_in, ERR_NULL_POINTER);

	unsafe {
		let rt = &mut *rt_ptr;
		let session = &mut *session_in;
		let node: NodeId;
		match id_type {
			1 => node = NodeId::new(0, id_u32).into(), //so works so far
			2 => node = NodeId::new(ns, cstr_to_string!(id_str)).into(),
			_ => return ERR_INVALID_TYPE,
		}
		//
		//let node = NodeId::new(0, id_u32).into(); //so works so far
		let r = rt.block_on(async { session.browse(&[hierarchical_desc(node)], 1000, None).await });
		match r {
			Ok(result) => {
				let it = &result[0];
				let refs = it.references.clone().unwrap_or_default();
				let n = refs.len() as i32;

				unsafe {
					//let n = count;
					// Assuming sizeof(Node) is equivalent to the size of the struct in Rust
					let ret_size = std::mem::size_of::<NodeAttribute>() * n as usize
						+ std::mem::size_of::<NodeHdl>();
					DSSetHandleSize(nodes, ret_size);

					(**nodes).dim_size = n;

					for i in 0..n as usize {
						let name = refs[i].browse_name.to_string();

						let name_cnt = name.len();
						let node_id_s = refs[i].node_id.node_id.identifier.to_string();

						//(**nodes).node_attribute[i].id = i as c_int;
						(**nodes).node_attribute[i].class = refs[i].node_class as u32 as c_int;

						(**nodes).node_attribute[i].display_name =
							DSNewHandle(name.len() + std::mem::size_of::<c_int>());
						(**nodes).node_attribute[i].node_uid =
							DSNewHandle(node_id_s.len() + std::mem::size_of::<c_int>());

						(**((**nodes).node_attribute[i].display_name)).cnt = name.len() as i32;
						(**((**nodes).node_attribute[i].node_uid)).cnt = node_id_s.len() as i32;

						let c_headers = match CString::new(name) {
							Ok(cs) => cs,
							Err(_) => return -1, // failed to convert to C string
						};
						MoveBlockChar(
							c_headers.as_ptr(), //seems to be OK, but 4 bytes shift
							(**((**nodes).node_attribute[i].display_name))
								.str
								.as_mut_ptr(),
							name_cnt,
						);

						let c_headers = match CString::new(node_id_s.to_string()) {
							Ok(cs) => cs,
							Err(_) => return -1, // failed to convert to C string
						};
						MoveBlockChar(
							c_headers.as_ptr(), //seems to be OK, but 4 bytes shift
							(**((**nodes).node_attribute[i].node_uid)).str.as_mut_ptr(),
							node_id_s.len(),
						);
					}
				}
				return n as i32;
			}

			Err(_) => {
				return ERR_BROWSE_ERROR;
			}
		}
	}
}

fn hierarchical_desc(node_id: NodeId) -> BrowseDescription {
	BrowseDescription {
		node_id,
		browse_direction: BrowseDirection::Forward,
		reference_type_id: ReferenceTypeId::HierarchicalReferences.into(),
		include_subtypes: true,
		node_class_mask: NodeClassMask::all().bits(),
		result_mask: BrowseResultMask::All as u32,
	}
}
