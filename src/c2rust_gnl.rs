#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]

use std::{
	clone::CloneToUninit,
	ffi::{c_char, c_int, c_ulong},
	os::fd::RawFd,
};
const ALLOC_SIZE: c_ulong = core::mem::size_of::<u8>() as c_ulong;
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

fn nl_position(buffer: &[u8]) -> Option<usize> {
	buffer.iter().position(|c| *c == b'\n')
}

unsafe extern "C" {
	fn malloc(_: c_ulong) -> *mut libc::c_void;
}

unsafe fn read_newln(
	fd: RawFd,
	count: &mut usize,
	static_buffer: &mut [u8; BUFFER_SIZE + 1],
	return_line: &mut Option<*mut u8>,
	call_number: usize,
) -> Option<*mut u8> {
	let mut read_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	let read_result = nix::unistd::read(fd, read_buffer.as_mut_slice());
	if read_result.is_err() || read_result.unwrap() == 0 && *count == 0 {
		static_buffer.fill(b'\0');
		return None;
	}
	match read_result.unwrap() {
		0 => {
			// EOF reached (other case already handled)
			let mut cur_retline = vec![0; *count + 1];
			static_buffer.as_slice().clone_into(&mut cur_retline);
			*return_line = Some(cur_retline.as_mut_ptr());
			std::mem::forget(cur_retline);
			static_buffer.fill(b'\0');
		}
		_ => {
			// read buffer has data
			*count += BUFFER_SIZE;
			#[cfg(debug_assertions)]
			eprintln!("{}: count++", call_number);
			if let Some(newline_pos) = nl_position(&read_buffer[..]) {
				let mut cur_retline = vec![0; *count + 1];
				static_buffer.as_slice().clone_into(&mut cur_retline);
				*return_line = Some(cur_retline.as_mut_ptr());
				std::mem::forget(cur_retline);
				static_buffer[..BUFFER_SIZE].copy_from_slice(&read_buffer[..]);
				static_buffer.copy_within(newline_pos + 1.., 0);
				static_buffer[(BUFFER_SIZE - newline_pos)..].fill(b'\0');
			} else {
				*return_line = read_newln(fd, count, static_buffer, return_line, call_number + 1);
			}
			if read_buffer.as_slice()[0] != b'\0' {
				// non-empty read buffer
				let cpy_from_read = {
					&read_buffer[..match nl_position(&read_buffer[..]) {
						Some(newline_idx) if newline_idx < BUFFER_SIZE => newline_idx + 1,
						_ => BUFFER_SIZE,
					}]
				};
				*count -= BUFFER_SIZE;
				cpy_from_read
					.as_ptr()
					.copy_to_nonoverlapping(return_line.unwrap().add(*count), cpy_from_read.len());
			}
		}
	}
	*return_line
}

///
/// read a line from a buffer into heap memory and return a pointer to the heap memory
/// this will never be called if the buffer is empty: `assert!(!&static_buffer.starts_with(&[0; BUFFER_SIZE + 1]));`
unsafe fn read_buffer(static_buffer: &mut [u8; BUFFER_SIZE + 1], count: usize) -> *mut c_char {
	let copy_return_line = malloc((count + 2) as c_ulong * ALLOC_SIZE) as *mut u8;
	if !copy_return_line.is_null() {
		static_buffer
			.as_ptr()
			.copy_to_nonoverlapping(copy_return_line, count + 1);
		*copy_return_line.add(count + 1) = b'\0';
	}
	// we know we have a newline in the buffer, we can just shift it
	debug_assert_eq!(static_buffer[count], b'\n');
	static_buffer.copy_within(count + 1.., 0);
	static_buffer[(BUFFER_SIZE - count)..].fill(b'\0');
	copy_return_line as *mut c_char
}

///
/// read a line from a file descriptor
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and calls foreign functions.
///
/// The caller must ensure that the `fd` is a valid file descriptor and that the buffer size is greater than 0.
///
/// The caller must free the returned pointer when it is no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(fd: RawFd) -> *mut c_char {
	if (BUFFER_SIZE as c_int) < 1 as c_int || !(0..=10240).contains(&fd) {
		return std::ptr::null_mut::<c_char>();
	}
	let fd: usize = fd as usize;
	static mut static_buffer: [[u8; BUFFER_SIZE + 1]; 10240] = [[0; BUFFER_SIZE + 1]; 10240];
	let mut count: usize = 0;
	while static_buffer[fd][count] != b'\0' && static_buffer[fd][count] != b'\n' {
		count += 1;
	}
	if count <= BUFFER_SIZE && static_buffer[fd][count] == b'\n' {
		return read_buffer(&mut (static_buffer[fd]), count);
	}
	if let Some(line) = read_newln(
		fd as RawFd,
		&mut count,
		&mut (static_buffer[fd]),
		&mut Option::None,
		0,
	) {
		let cstr_line = std::ffi::CStr::from_ptr(line as *const i8);
		let copy_return_line =
			malloc((cstr_line.count_bytes() + 1) as c_ulong * ALLOC_SIZE) as *mut u8;
		if !copy_return_line.is_null() {
			cstr_line.clone_to_uninit(copy_return_line);
		}
		copy_return_line as *mut c_char
	} else {
		std::ptr::null_mut::<c_char>()
	}
}
