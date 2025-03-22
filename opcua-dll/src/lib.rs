#![allow(unused_unsafe)] //fom unsafe macros in unsafe code
//==============================================================================
//
// Title:		OPC UA Wrapper for LabVIEW
// Purpose:		Proof of the concept/Feasibility Study
//
// Created on:	08-MAR-2025 by AD.
// License:     MPL-2.0
// (based on https://github.com/FreeOpcUa/async-opcua)
//==============================================================================
pub mod errors;
#[macro_use]
pub mod labview; // common functions and structures
pub mod browser;
pub mod client;
pub mod client_variables;
pub mod runtime;
pub mod server; //tokio helper
pub mod server_variables;
