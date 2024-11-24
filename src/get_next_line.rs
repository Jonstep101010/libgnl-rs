#![allow(static_mut_refs)]

const BUF_USIZE: usize = 16;
const BUF_SIZE_ONE: usize = BUF_USIZE + 1;

use ::libc;
use libft_rs::{ft_calloc::ft_calloc, ft_strlcpy::ft_strlcpy};
extern "C" {
	fn free(_: *mut libc::c_void);
	fn read(__fd: libc::c_int, __buf: *mut libc::c_void, __nbytes: size_t) -> ssize_t;
}

///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// Takes a string, a character, and a maximum length.
/// Returns the index of the character in the string.
/// If the character is not found, returns the maximum length.
#[no_mangle]
pub unsafe extern "C" fn index_of(str: *mut libc::c_char, max_len: usize) -> usize {
	let mut i: usize = 0;
	while i < max_len
		&& libc::c_int::from(*str.add(i)) != '\n' as libc::c_int
		&& libc::c_int::from(*str.add(i)) != '\0' as i32
	{
		i += 1;
	}
	i
	// same as:
	// CStr::from_ptr(str)
	// 	.to_bytes_with_nul()
	// 	.iter()
	// 	.position(|&c| c == b'\n' || c == b'\0')
	// 	.map(|pos| if pos < max_len { pos } else { max_len })
	// 	.unwrap_or(max_len)
}

pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;
#[allow(unused_mut)]
unsafe fn check_free(
	mut buf: *mut libc::c_char,
	buf_idx: usize,
	mut line: *mut libc::c_char,
	is_buf: bool,
) -> *mut libc::c_char {
	if line.is_null() {
		return std::ptr::null_mut::<libc::c_char>();
	}
	if is_buf {
		let mut buf_nl_idx: usize = index_of(buf, 2_147_483_647);
		std::ptr::copy(buf, line, buf_idx + 1);
		if libc::c_int::from(*buf.add(buf_nl_idx)) == '\n' as i32 {
			buf_nl_idx += 1;
		} else {
			*buf.add(buf_nl_idx) = libc::c_char::try_from(0 as libc::c_int).unwrap();
		}
		std::ptr::copy(
			buf.add(buf_nl_idx) as *const libc::c_void,
			buf.cast::<libc::c_void>(),
			BUF_USIZE - buf_nl_idx + 1,
		);
	}
	let mut gnl_idx: usize = index_of(line, 2_147_483_647);
	if libc::c_int::from(*line.add(gnl_idx)) == '\n' as i32 {
		gnl_idx += 1;
	}
	let mut ret: *mut libc::c_char = ft_calloc(
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		(gnl_idx + 1) as size_t,
	)
	.cast::<libc::c_char>();
	if ret.is_null() {
		free(line.cast::<libc::c_void>());
		return std::ptr::null_mut::<libc::c_char>();
	}
	std::ptr::copy(line, ret, gnl_idx);
	free(line.cast::<libc::c_void>());
	ret
}

///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// Takes a file descriptor.
/// Returns the next line from the file descriptor.
/// If the file descriptor is invalid, returns a null pointer.
/// If the line is empty, returns a null pointer.
#[no_mangle]
pub unsafe extern "C" fn get_next_line(fd: libc::c_int) -> *mut libc::c_char {
	static mut buf: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	if fd < 0 as libc::c_int || BUF_USIZE < 1 {
		return std::ptr::null_mut::<libc::c_char>();
	}
	let mut line: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
	let mut buf_idx: usize = 0;
	loop {
		if !(buf_idx < BUF_USIZE && libc::c_int::from(buf[buf_idx]) != 0) {
			break;
		}
		if libc::c_int::from(buf[buf_idx]) == '\n' as i32 {
			line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				BUF_SIZE_ONE as size_t,
			)
			.cast::<libc::c_char>();
			if line.is_null() {
				return std::ptr::null_mut::<libc::c_char>();
			}
			return check_free(buf.as_mut_ptr(), buf_idx, line, true);
		}
		buf_idx += 1;
	}
	if libc::c_int::from(buf[buf_idx]) != '\n' as i32 {
		read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
	}
	check_free(buf.as_mut_ptr(), buf_idx, line, false)
}

#[allow(unused_mut)]
unsafe extern "C" fn read_line(
	mut buf: *mut libc::c_char,
	fd: libc::c_int,
	mut buf_idx: *mut usize,
	mut line: *mut *mut libc::c_char,
) -> *mut libc::c_char {
	let mut tmp: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	tmp.as_mut_ptr().write_bytes(0, BUF_USIZE);
	let rd: ssize_t = read(
		fd,
		tmp.as_mut_ptr().cast::<libc::c_void>(),
		BUF_USIZE as size_t,
	);
	if rd == -1 {
		buf.write_bytes(0, BUF_USIZE);
		return buf.cast::<libc::c_char>();
	}
	if rd > 0 {
		*buf_idx += BUF_USIZE;
	}
	let mut tmp_nl_idx: usize = index_of(tmp.as_mut_ptr(), BUF_USIZE);
	if (libc::c_int::from(tmp[tmp_nl_idx]) == '\n' as i32 || rd == 0 && *buf_idx != 0)
		&& !{
			*line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				(*buf_idx + 1) as size_t,
			)
			.cast::<libc::c_char>();
			if (*line).is_null() {
				false
			} else {
				ft_strlcpy(*line, buf, (*buf_idx + 1) as size_t);
				std::ptr::copy(
					tmp.as_mut_ptr() as *const libc::c_void,
					buf.cast::<libc::c_void>(),
					BUF_USIZE,
				);
				let mut buf_nl_idx: usize = index_of(buf, BUF_USIZE + 1);
				if libc::c_int::from(*buf.add(buf_nl_idx)) == '\n' as i32 {
					buf_nl_idx += 1;
				} else {
					*buf.add(buf_nl_idx) = libc::c_char::try_from(0 as libc::c_int).unwrap();
				}
				std::ptr::copy(
					buf.add(buf_nl_idx) as *const libc::c_void,
					buf.cast::<libc::c_void>(),
					BUF_USIZE - buf_nl_idx + 1,
				);
				true
			}
		} {
		return std::ptr::null_mut::<libc::c_char>();
	}
	if libc::c_int::from(tmp[tmp_nl_idx]) != '\n' as i32
		&& rd != 0
		&& (read_line(buf, fd, buf_idx, line)).is_null()
	{
		return std::ptr::null_mut::<libc::c_char>();
	}
	if rd > 0 && *buf_idx != 0 {
		*buf_idx -= BUF_USIZE;
		tmp_nl_idx = index_of(tmp.as_mut_ptr(), BUF_USIZE);
		std::ptr::copy(tmp.as_mut_ptr(), (*line).add(*buf_idx), tmp_nl_idx);
		if libc::c_int::from(tmp[tmp_nl_idx]) == '\n' as i32 {
			*(*line).add(*buf_idx + tmp_nl_idx) = libc::c_char::try_from('\n' as i32).unwrap();
		}
	}
	*line
}

#[cfg(test)]
mod tests {
	use super::*;
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

			let path = CString::new("test.txt").unwrap();
			let fd = libc::open(path.as_ptr(), libc::O_RDONLY);
			let mut line: *mut libc::c_char = get_next_line(fd);
			let mut my_str = String::new();
			while !line.is_null() {
				let line_str = std::ffi::CStr::from_ptr(line).to_str().unwrap();
				// read to rust string
				my_str.push_str(line_str);
				// free c line
				libc::free(line.cast::<libc::c_void>());
				line = get_next_line(fd);
			}
			let expected = std::fs::read_to_string("test.txt").unwrap();
			assert_eq!(expected, my_str);
		}
		let mut logfile = std::fs::File::create("log.txt").unwrap();
		std::fs::read_to_string("test.txt")
			.unwrap()
			.lines()
			.for_each(|line| {
				// write to log file
				logfile.write_all(line.as_bytes()).unwrap();
				println!("{}", line);
			});
		// diff against expected.txt

		let expected = std::fs::read_to_string("expected.txt").unwrap();
		let output = std::fs::read_to_string("log.txt").unwrap();
		assert_eq!(output, expected);
		std::fs::remove_file("log.txt").unwrap();
	}
}
