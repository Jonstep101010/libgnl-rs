use get_next_line::c2rust_gnl::get_next_line;
use std::io::Write;

///
/// running with miri:
/// set -x MIRIFLAGS "-Zmiri-disable-isolation -Zmiri-ignore-leaks"
/// cargo +nightly miri test
#[test]
fn gnl_basic() {
	// open a file and get its file descriptor, print contents to stdout using get_next_line(fd)
	// in c:
	// int fd = open("../test.txt", O_RDONLY);
	// loop over output of get_next_line(fd) and print to terminal
	//

	unsafe {
		use std::ffi::CString;

		let path = CString::new("tests/test.txt").unwrap();
		let fd = libc::open(path.as_ptr(), libc::O_RDONLY);
		let mut line: *mut libc::c_char = get_next_line::c2rust_gnl::get_next_line(fd);
		let mut my_str = String::new();
		while !line.is_null() {
			let line_str = std::ffi::CStr::from_ptr(line).to_str().unwrap();
			// read to rust string
			my_str.push_str(line_str);
			// free c line
			libc::free(line.cast::<libc::c_void>());
			line = get_next_line(fd);
		}
		// assert!(false);
		let expected = std::fs::read_to_string("tests/test.txt").unwrap();
		assert_eq!(expected, my_str);
	}
	let mut logfile = std::fs::File::create("log.txt").unwrap();
	std::fs::read_to_string("tests/test.txt")
		.unwrap()
		.lines()
		.for_each(|line| {
			// write to log file
			logfile.write_all(line.as_bytes()).unwrap();
			println!("{}", line);
		});
	// diff against expected.txt

	let expected = std::fs::read_to_string("tests/expected.txt").unwrap();
	let output = std::fs::read_to_string("log.txt").unwrap();
	assert_eq!(output, expected);
	std::fs::remove_file("log.txt").unwrap();
}
