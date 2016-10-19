extern crate serde_codegen;

use std::env;
use std::path::Path;

pub fn main() {
	let out_dir = env::var_os("OUT_DIR").unwrap();

	let srcs: [&'static str; 2] = [ "admin_types", "speedruncom_types" ];

	for src in srcs.into_iter() {
		serde_codegen::expand(
			&Path::new(&format!("src/modules/{}.in.rs", src)),
			&Path::new(&out_dir).join(&format!("{}.rs", src))
		).unwrap();
	}
}
