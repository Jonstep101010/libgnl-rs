// build.rs
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
	// add unix dependencies below
	// println!("cargo:rustc-flags=-l readline");
}

#[cfg(target_os = "macos")]
fn main() {
	// Retrieve the BUFFER_SIZE environment variable, default to 3 if not set

	use std::path::Path;
	let buffer_size = env::var("BUFFER_SIZE").unwrap_or_else(|_| "3".to_string());

	// Validate and convert the buffer size to a valid usize
	let buffer_size: usize = buffer_size.parse().unwrap_or_else(|_| {
		eprintln!("Invalid BUFFER_SIZE value, defaulting to 3");
		3
	});

	// Write the BUFFER_SIZE to a file that can be included in the Rust code
	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("buffer_size.rs");
	let mut file = File::create(&dest_path).unwrap();
	writeln!(file, "pub const BUFFER_SIZE: usize = {};", buffer_size).unwrap();

	let cargo_toml_path = Path::new("Cargo.toml");
	let mut cargo_toml = String::new();
	File::open(cargo_toml_path)
		.unwrap()
		.read_to_string(&mut cargo_toml)
		.unwrap();
	// set crate-type to rlib (prefix non-commented out, comment out static-lib in Cargo.toml)
	#[cfg(test)]
	let new_cargo_toml = cargo_toml.replace("crate-type = [\"staticlib\"]", "crate-type = [\"rlib\"]");

	// set crate-type to static-lib (prefix non-commented out, comment out rlib in Cargo.toml)
	#[cfg(not(test))]
	let new_cargo_toml = cargo_toml.replace("crate-type = [\"rlib\"]", "crate-type = [\"staticlib\"]");
	// Write the updated Cargo.toml back to the file

	let mut file = std::fs::OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(cargo_toml_path)
		.unwrap();
	file.write_all(new_cargo_toml.as_bytes()).unwrap();

	println!("cargo:rerun-if-env-changed=BUFFER_SIZE");
}
