#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]

use std::{
	clone::CloneToUninit,
	ffi::{CStr, CString, c_char, c_int, c_ulong},
	fmt::format,
	io::Read,
	os::fd::RawFd,
	ptr::slice_from_raw_parts_mut,
};
const ALLOC_SIZE: c_ulong = core::mem::size_of::<u8>() as c_ulong;
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

unsafe extern "C" {
	fn malloc(_: c_ulong) -> *mut libc::c_void;
}

fn shift_static_buffer(static_buffer: &mut [u8]) {
	match static_buffer.iter().position(|&c| c == b'\n') {
		Some(idx) => {
			static_buffer.copy_within(idx + 1.., 0);
			static_buffer[(BUFFER_SIZE - idx)..].fill(b'\0');
		}
		None => {
			static_buffer.fill(b'\0');
		}
	};
}

fn alloc_newline(
	count: usize,
	static_buffer: &mut [u8; BUFFER_SIZE + 1],
	return_line: &mut Option<*mut u8>,
	read_buffer: &mut [u8; BUFFER_SIZE],
) -> Option<*mut u8> {
	let mut alloc = vec![0; count + 1];
	static_buffer.as_slice().clone_into(&mut alloc);
	*return_line = Some(alloc.as_mut_ptr());
	std::mem::forget(alloc);
	static_buffer[..BUFFER_SIZE].copy_from_slice(&read_buffer[..]);
	shift_static_buffer(static_buffer);
	*return_line
}

#[unsafe(no_mangle)]
unsafe fn read_newln(
	fd: RawFd,
	count: &mut usize,
	static_buffer: &mut [u8; BUFFER_SIZE + 1],
	return_line: &mut Option<*mut u8>,
) -> Option<*mut u8> {
	let mut read_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	let read_result = nix::unistd::read(fd, read_buffer.as_mut_slice());
	if read_result.is_err() || read_result.unwrap() == 0 && *count == 0 {
		static_buffer.fill(b'\0');
		return None;
	}
	match read_result.unwrap() {
		0 if *count != 0 => {
			// EOF reached
			alloc_newline(*count, static_buffer, return_line, &mut read_buffer)?;
		}
		_ => {
			// read buffer has data
			*count += BUFFER_SIZE;
			if let Some(_newline_pos) = read_buffer.as_slice().iter().position(|&c| c == b'\n') {
				alloc_newline(*count, static_buffer, return_line, &mut read_buffer)?;
			} else {
				*return_line = read_newln(fd, count, static_buffer, return_line);
			}
		}
	}
	if read_buffer.as_slice()[0] != b'\0' {
		let cpy_from_read =
			&read_buffer.as_slice()[..match read_buffer.as_slice().iter().position(|&c| c == b'\n')
			{
				None => BUFFER_SIZE,
				Some(newline_idx) => {
					if newline_idx < BUFFER_SIZE {
						newline_idx + 1
					} else {
						BUFFER_SIZE + 1
					}
				}
			}];
		*count -= BUFFER_SIZE;
		dbg!(CStr::from_bytes_with_nul_unchecked(cpy_from_read));
		let slice_ret =
			slice_from_raw_parts_mut(return_line.unwrap(), *count + cpy_from_read.len());
		(*slice_ret)[*count..].copy_from_slice(cpy_from_read);
		dbg!(CStr::from_bytes_with_nul_unchecked(&*slice_ret));
		*return_line = Some(slice_ret.as_mut_ptr());
	}
	*return_line
}

fn read_buffer(static_buffer: &mut [u8; BUFFER_SIZE + 1]) -> Option<*mut u8> {
	let mut line_staticbuffer: Vec<u8> = vec![0; BUFFER_SIZE + 1];
	let newline_idx = static_buffer
		.as_slice()
		.iter()
		.position(|&c| c == b'\n')
		.expect("newline has to be present in the buffer")
		+ 1;
	let static_buffer_slice = &static_buffer.as_slice()[..newline_idx];
	let ptr = line_staticbuffer.as_mut_ptr();
	let destination_slice = unsafe { std::slice::from_raw_parts_mut(ptr, newline_idx) };
	destination_slice.copy_from_slice(static_buffer_slice);
	std::mem::forget(line_staticbuffer);
	shift_static_buffer(static_buffer.as_mut_slice());
	Some(ptr)
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
	let return_line = if count <= BUFFER_SIZE && static_buffer[fd][count] == b'\n' {
		read_buffer(&mut (static_buffer[fd]))
	} else {
		read_newln(
			fd as RawFd,
			&mut count,
			&mut (static_buffer[fd]),
			&mut Option::None,
		)
	};
	if let Some(line) = return_line {
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
