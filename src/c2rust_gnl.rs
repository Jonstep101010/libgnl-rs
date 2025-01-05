#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]

use std::{
	ffi::{c_char, c_int, c_ulong},
	os::fd::RawFd,
	ptr::drop_in_place,
};
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

unsafe extern "C" {
	fn malloc(_: c_ulong) -> *mut libc::c_void;
	fn calloc(_: c_ulong, _: c_ulong) -> *mut libc::c_void;
	// fn free(_: *mut libc::c_void);
	// fn strchr(_: *const c_char, _: c_int) -> *mut c_char;
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

#[allow(unused_mut)]
#[unsafe(no_mangle)]
unsafe fn terminated_line_copy(mut return_line: Option<*mut u8>) -> *mut c_char {
	if return_line.is_none() {
		return std::ptr::null_mut::<c_char>();
	}
	// let len = strlen(return_line);
	let len = std::ffi::CStr::from_ptr(return_line.unwrap() as *const i8).count_bytes();
	let mut copy_return_line =
		malloc((len + 1).wrapping_mul(::core::mem::size_of::<u8>()) as c_ulong) as *mut u8;
	if !copy_return_line.is_null() {
		copy_return_line.copy_from_nonoverlapping(return_line.unwrap(), len + 1);
	}
	// free(return_line as *mut libc::c_void);
	drop_in_place(return_line.unwrap());
	copy_return_line as *mut c_char
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
	unsafe fn alloc_newline(
		count: usize,
		static_buffer: &mut [u8; BUFFER_SIZE + 1],
		return_line: &mut Option<*mut u8>,
		read_buffer: *const u8,
	) -> Option<*mut u8> {
		let alloc = calloc(
			(count + 1) as c_ulong,
			::core::mem::size_of::<c_char>() as c_ulong,
		) as *mut u8;
		if alloc.is_null() {
			return None;
		}
		// copy the length of the terminated charptr into return_line (allocated)
		alloc.copy_from_nonoverlapping(
			static_buffer.as_ptr(),
			libc::strlen(static_buffer.as_ptr() as *const i8) as usize,
		);
		// copy length - 1 of read buffer into static_buffer
		static_buffer
			.as_mut_ptr()
			.copy_from_nonoverlapping(read_buffer, BUFFER_SIZE);
		shift_static_buffer(static_buffer);
		*return_line = Some(alloc);
		*return_line
	}
	if read_result.is_err() || read_result.unwrap() == 0 && *count == 0 {
		static_buffer.fill(b'\0');
		return None;
	}
	match read_result.unwrap() {
		0 if *count != 0 => {
			// EOF reached
			alloc_newline(*count, static_buffer, return_line, read_buffer.as_ptr())?;
		}
		_ => {
			// read buffer has data
			*count += BUFFER_SIZE;
			if let Some(_newline_pos) = read_buffer.as_slice().iter().position(|&c| c == b'\n') {
				alloc_newline(*count, static_buffer, return_line, read_buffer.as_ptr())?;
			} else {
				*return_line = read_newln(fd, count, static_buffer, return_line);
			}
		}
	}
	if (*read_buffer.as_ptr()) != b'\0' {
		*count -= BUFFER_SIZE;
		return_line.unwrap().add(*count).copy_from_nonoverlapping(
			read_buffer.as_ptr(),
			match read_buffer.as_slice().iter().position(|&c| c == b'\n') {
				None => BUFFER_SIZE,
				Some(newline_idx) => {
					if newline_idx < BUFFER_SIZE {
						newline_idx + 1
					} else {
						BUFFER_SIZE + 1
					}
				}
			},
		);
	}
	*return_line
}

#[unsafe(no_mangle)]
unsafe fn read_buffer(static_buffer: &mut [u8; BUFFER_SIZE + 1]) -> Option<*mut u8> {
	let line_staticbuffer: *mut c_char = calloc(
		(BUFFER_SIZE + 1) as c_ulong,
		::core::mem::size_of::<c_char>() as c_ulong,
	) as *mut c_char;
	if line_staticbuffer.is_null() {
		return None;
	}
	let newline_idx = static_buffer
		.as_slice()
		.iter()
		.position(|&c| c == b'\n')
		.unwrap();
	line_staticbuffer.copy_from_nonoverlapping(static_buffer.as_ptr() as *mut i8, newline_idx + 1);
	shift_static_buffer(static_buffer.as_mut_slice());
	Some(line_staticbuffer as *mut u8)
}

///
/// read a line from a file descriptor
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers and calls foreign functions.
/// The caller must ensure that the `fd` is a valid file descriptor and that the buffer size is greater than 0.
/// The caller must free the returned pointer when it is no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(fd: RawFd) -> *mut c_char {
	if (BUFFER_SIZE as c_int) < 1 as c_int || !(0..=10240).contains(&fd) {
		return std::ptr::null_mut::<c_char>();
	}
	let fd: usize = fd as usize;
	static mut static_buffer: [[u8; BUFFER_SIZE + 1]; 10240] = [[0; BUFFER_SIZE + 1]; 10240];
	let mut count: usize = 0;
	while static_buffer[fd][count] as c_int != '\0' as i32
		&& static_buffer[fd][count] as c_int != '\n' as i32
	{
		count += 1;
	}
	let mut return_line: Option<*mut u8> = None;
	terminated_line_copy(
		if count <= BUFFER_SIZE && static_buffer[fd][count] as c_int == '\n' as i32 {
			read_buffer(&mut (static_buffer[fd]))
		} else {
			read_newln(
				fd as RawFd,
				&mut count,
				&mut (static_buffer[fd]),
				&mut return_line,
			)
		},
	)
}
