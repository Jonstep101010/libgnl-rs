#![allow(static_mut_refs)]

const BUF_SIZE: i32 = 16;
const BUF_SIZE_ONE: usize = BUF_SIZE as usize + 1;

use ::libc;
use libft_rs::{ft_calloc::ft_calloc, ft_memset::ft_memset, ft_strlcpy::ft_strlcpy};
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
pub unsafe extern "C" fn index_of(str: *mut libc::c_char, max_len: libc::c_int) -> libc::c_int {
	let mut i: libc::c_int = 0;
	while i < max_len
		&& libc::c_int::from(*str.offset(i as isize)) != '\n' as libc::c_int
		&& libc::c_int::from(*str.offset(i as isize)) != '\0' as i32
	{
		i += 1;
	}
	i
}

pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;
#[allow(unused_mut)]
unsafe extern "C" fn check_free(
	mut buf: *mut libc::c_char,
	buf_idx: libc::c_int,
	mut line: *mut libc::c_char,
	is_buf: bool,
) -> *mut libc::c_char {
	if line.is_null() {
		return std::ptr::null_mut::<libc::c_char>();
	}
	if is_buf {
		let mut buf_nl_idx: libc::c_int = index_of(buf, 2_147_483_647 as libc::c_int);
		std::ptr::copy(buf, line, (buf_idx + 1).try_into().unwrap());
		if libc::c_int::from(*buf.offset(buf_nl_idx as isize)) == '\n' as i32 {
			buf_nl_idx += 1;
		} else {
			*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
		}
		std::ptr::copy(
			buf.offset(buf_nl_idx as isize) as *const libc::c_void,
			buf as *mut libc::c_void,
			(BUF_SIZE - buf_nl_idx + 1 as libc::c_int)
				.try_into()
				.unwrap(),
		);
	}
	let mut gnl_idx: libc::c_int = index_of(line, 2_147_483_647 as libc::c_int);
	if libc::c_int::from(*line.offset(gnl_idx as isize)) == '\n' as i32 {
		gnl_idx += 1;
	}
	let mut ret: *mut libc::c_char = ft_calloc(
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		(gnl_idx + 1 as libc::c_int) as size_t,
	) as *mut libc::c_char;
	if ret.is_null() {
		free(line as *mut libc::c_void);
		return std::ptr::null_mut::<libc::c_void>() as *mut libc::c_char;
	}
	std::ptr::copy(line, ret, gnl_idx.try_into().unwrap());
	free(line as *mut libc::c_void);
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
	if fd < 0 as libc::c_int || BUF_SIZE < 1 as libc::c_int {
		return std::ptr::null_mut::<libc::c_char>();
	}
	let mut line: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
	let mut buf_idx: libc::c_int = -(1 as libc::c_int);
	loop {
		buf_idx += 1;
		if !(buf_idx < BUF_SIZE && libc::c_int::from(buf[buf_idx as usize]) != 0) {
			break;
		}
		if libc::c_int::from(buf[buf_idx as usize]) == '\n' as i32 {
			line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				(BUF_SIZE + 1 as libc::c_int) as size_t,
			) as *mut libc::c_char;
			if line.is_null() {
				return std::ptr::null_mut::<libc::c_char>();
			}
			return check_free(buf.as_mut_ptr(), buf_idx, line, true);
		}
	}
	if libc::c_int::from(buf[buf_idx as usize]) != '\n' as i32 {
		read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
	}
	check_free(buf.as_mut_ptr(), buf_idx, line, false)
}

#[allow(unused_mut)]
unsafe extern "C" fn read_line(
	mut buf: *mut libc::c_char,
	fd: libc::c_int,
	mut buf_idx: *mut libc::c_int,
	mut line: *mut *mut libc::c_char,
) -> *mut libc::c_char {
	let mut tmp: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	let rd: libc::c_int = read(
		fd,
		ft_memset(
			tmp.as_mut_ptr() as *mut libc::c_void,
			0 as libc::c_int,
			BUF_SIZE as size_t,
		),
		BUF_SIZE as size_t,
	) as libc::c_int;
	if rd == -(1 as libc::c_int) {
		return ft_memset(
			buf as *mut libc::c_void,
			0 as libc::c_int,
			BUF_SIZE as size_t,
		) as *mut libc::c_char;
	}
	if rd > 0 as libc::c_int {
		*buf_idx += BUF_SIZE;
	}
	let mut tmp_nl_idx: libc::c_int = index_of(tmp.as_mut_ptr(), BUF_SIZE);
	if (libc::c_int::from(tmp[tmp_nl_idx as usize]) == '\n' as i32
		|| rd == 0 as libc::c_int && *buf_idx != 0 as libc::c_int)
		&& !{
			let mut line = line;
			let mut buf = buf;
			let mut tmp = tmp.as_mut_ptr();
			let buf_idx = *buf_idx;
			*line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				(buf_idx + 1 as libc::c_int) as size_t,
			) as *mut libc::c_char;
			if (*line).is_null() {
				false
			} else {
				ft_strlcpy(*line, buf, (buf_idx + 1 as libc::c_int) as size_t);
				std::ptr::copy(
					tmp as *const libc::c_void,
					buf as *mut libc::c_void,
					BUF_SIZE.try_into().unwrap(),
				);
				let mut buf_nl_idx: libc::c_int = index_of(buf, BUF_SIZE + 1 as libc::c_int);
				if libc::c_int::from(*buf.offset(buf_nl_idx as isize)) == '\n' as i32 {
					buf_nl_idx += 1;
				} else {
					*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
				}
				std::ptr::copy(
					buf.offset(buf_nl_idx as isize) as *const libc::c_void,
					buf as *mut libc::c_void,
					(BUF_SIZE - buf_nl_idx + 1 as libc::c_int)
						.try_into()
						.unwrap(),
				);
				true
			}
		} {
		return std::ptr::null_mut::<libc::c_char>();
	}
	if libc::c_int::from(tmp[tmp_nl_idx as usize]) != '\n' as i32
		&& rd != 0 as libc::c_int
		&& (read_line(buf, fd, buf_idx, line)).is_null()
	{
		return std::ptr::null_mut::<libc::c_char>();
	}
	if rd > 0 as libc::c_int && *buf_idx != 0 as libc::c_int {
		*buf_idx -= BUF_SIZE;
		tmp_nl_idx = index_of(tmp.as_mut_ptr(), BUF_SIZE);
		std::ptr::copy(
			tmp.as_mut_ptr(),
			(*line).offset(*buf_idx as isize),
			tmp_nl_idx.try_into().unwrap(),
		);
		if libc::c_int::from(tmp[tmp_nl_idx as usize]) == '\n' as i32 {
			*(*line).offset((*buf_idx + tmp_nl_idx) as isize) = '\n' as i32 as libc::c_char;
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
				libc::free(line as *mut libc::c_void);
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
