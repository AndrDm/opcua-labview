//==============================================================================
//
// Title:		Build Script for opcua
// Purpose:		Dynamic link with labview lib from LabVIEW 2025-2017.
//
// Created on:	15-MAR-2025 at 22:37:46 by AD.
//
//==============================================================================

use std::{env, path::PathBuf};
extern crate winres;

fn main() {
	let bitness = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

	if bitness == "x86" {
		match find_cintools_folder_32() {
			Some(path) => println!("cargo::rustc-link-search={}", path.display()),
			None => println!("Cintools (32-bit) folder not found"),
		}
	} else if bitness == "x86_64" {
		match find_cintools_folder_64() {
			Some(path) => println!("cargo::rustc-link-search={}", path.display()),
			None => println!("Cintools (64-bit) folder not found"),
		}
	} else {
		println!("bitness unknown");
	}

	println!("cargo:rustc-link-lib=labview"); //without .lib!
	println!("cargo:rustc-link-lib=user32");

	let res = winres::WindowsResource::new();
	res.compile().unwrap();
}
//==============================================================================
// Helper functions
//

fn find_cintools_folder_64() -> Option<PathBuf> {
	for year in (2017..=2025).rev() {
		let folder_path = PathBuf::from(format!(
			"C:\\Program Files\\National Instruments\\LabVIEW {}\\cintools",
			year
		));
		if folder_path.is_dir() {
			return Some(folder_path);
		}
	}
	None
}

fn find_cintools_folder_32() -> Option<PathBuf> {
	for year in (2017..=2025).rev() {
		let folder_path = PathBuf::from(format!(
			"C:\\Program Files (x86)\\National Instruments\\LabVIEW {}\\cintools",
			year
		));
		if folder_path.is_dir() {
			return Some(folder_path);
		}
	}
	None
}
