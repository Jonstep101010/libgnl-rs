#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs)]

use std::{
	ffi::{c_char, c_int, c_ulong},
	os::fd::RawFd,
	ptr::drop_in_place,
};
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

unsafe extern "C" {
	pub type __sFILEX;
	fn malloc(_: c_ulong) -> *mut libc::c_void;
	fn calloc(_: c_ulong, _: c_ulong) -> *mut libc::c_void;
	// fn free(_: *mut libc::c_void);
	// fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: c_ulong) -> *mut libc::c_void;
	// fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: c_ulong)
	// -> *mut libc::c_void;
	// fn memset(_: *mut libc::c_void, _: c_int, _: c_ulong) -> *mut libc::c_void;
	fn strchr(_: *const c_char, _: c_int) -> *mut c_char;
	// fn strlen(_: *const c_char) -> c_ulong;
	// fn bzero(_: *mut libc::c_void, _: c_ulong);
	// only used if building with main
	// fn getcwd(_: *mut c_char, _: size_t) -> *mut c_char;
	// fn open(_: *const c_char, _: c_int, _: ...) -> c_int;
	// static mut __stderrp: *mut FILE;
	// fn fprintf(_: *mut FILE, _: *const c_char, _: ...) -> c_int;
}
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = c_ulong;
pub type __darwin_ssize_t = libc::c_long;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type ssize_t = __darwin_ssize_t;
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: c_int,
	pub _w: c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: c_int,
	pub _cookie: *mut libc::c_void,
	pub _close: Option<unsafe extern "C" fn(*mut libc::c_void) -> c_int>,
	pub _read: Option<unsafe extern "C" fn(*mut libc::c_void, *mut c_char, c_int) -> c_int>,
	pub _seek: Option<unsafe extern "C" fn(*mut libc::c_void, fpos_t, c_int) -> fpos_t>,
	pub _write: Option<unsafe extern "C" fn(*mut libc::c_void, *const c_char, c_int) -> c_int>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: c_int,
	pub _offset: fpos_t,
}
pub type FILE = __sFILE;
#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
unsafe fn shift_static_buffer(static_buffer: *mut u8) {
	let newline_pos: *const c_char = strchr(static_buffer as *const c_char, '\n' as i32);
	if newline_pos.is_null() {
		static_buffer.write_bytes(b'\0', BUFFER_SIZE);
	} else {
		// get index after '\n'
		let start = newline_pos
			.offset_from(static_buffer as *const i8)
			.wrapping_add(1);
		let shift_len: usize = (BUFFER_SIZE + 1).wrapping_sub_signed(start);
		// shift contents from after '\n' to beginning
		static_buffer.copy_from(static_buffer.offset(start), shift_len);
		static_buffer
			.add(shift_len)
			.write_bytes(b'\0', (BUFFER_SIZE).wrapping_sub(shift_len));
	}
}
#[allow(unused_mut)]
#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
unsafe fn terminated_line_copy(mut return_line: Option<*mut u8>) -> *mut c_char {
	if return_line.is_none() {
		return std::ptr::null_mut::<c_char>();
	}
	// let len = strlen(return_line);
	let len = std::ffi::CStr::from_ptr(return_line.unwrap() as *const i8).count_bytes();
	let mut copy_return_line: *mut c_char =
		malloc((len + 1).wrapping_mul(::core::mem::size_of::<c_char>()) as size_t) as *mut c_char;
	if !copy_return_line.is_null() {
		copy_return_line.copy_from_nonoverlapping(return_line.unwrap() as *const i8, len + 1);
	}
	// free(return_line as *mut libc::c_void);
	drop_in_place(return_line.unwrap());
	copy_return_line
}

///
/// returns: `return_line`
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn copy_into_return_line(
	count: &mut usize,
	return_line: *mut u8,
	temp_buffer: *const u8,
) -> *mut u8 {
	if *temp_buffer != b'\0' {
		*count -= BUFFER_SIZE;
		let newline: *const u8 = strchr(temp_buffer as *const i8, '\n' as i32) as *const u8;
		let len = match newline.is_null() {
			true => BUFFER_SIZE,
			false => {
				if (newline.offset_from(temp_buffer)) < BUFFER_SIZE as isize {
					newline.offset_from(temp_buffer) as usize + 1
				} else {
					BUFFER_SIZE + 1
				}
			}
		};
		return_line
			.add(*count)
			.copy_from_nonoverlapping(temp_buffer, len);
	}
	return_line
}
#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
unsafe fn read_newln(
	fd: RawFd,
	count: &mut usize,
	static_buffer: *mut u8,
	return_line: &mut Option<*mut u8>,
) -> Option<*mut u8> {
	let mut temp_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	let read_result = nix::unistd::read(fd, temp_buffer.as_mut_slice());
	unsafe fn alloc_newline(
		count: usize,
		static_buffer: *mut u8,
		return_line: &mut Option<*mut u8>,
		temp_buffer: *const u8,
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
			static_buffer,
			libc::strlen(static_buffer as *const i8) as usize,
		);
		// copy length - 1 of read buffer into static_buffer
		static_buffer.copy_from_nonoverlapping(temp_buffer, BUFFER_SIZE);
		shift_static_buffer(static_buffer);
		*return_line = Some(alloc);
		*return_line
	}
	if read_result.is_err() || read_result.unwrap() == 0 && *count == 0 {
		static_buffer.write_bytes(0, BUFFER_SIZE + 1);
		return None;
	}
	match read_result.unwrap() {
		0 if *count != 0 => {
			// EOF reached
			alloc_newline(*count, static_buffer, return_line, temp_buffer.as_ptr())?;
		}
		_ => {
			// read buffer has data
			*count += BUFFER_SIZE;
			let newline_pos = strchr(temp_buffer.as_ptr() as *const i8, '\n' as i32);
			if !newline_pos.is_null()
				&& alloc_newline(*count, static_buffer, return_line, temp_buffer.as_ptr()).is_none()
			{
				return None;
			}
			if newline_pos.is_null() {
				*return_line = read_newln(fd, count, static_buffer, return_line);
			}
		}
	}
	Some(copy_into_return_line(
		count,
		return_line.unwrap(),
		temp_buffer.as_ptr(),
	))
}

#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
unsafe fn read_buffer(static_buffer: *mut u8) -> Option<*mut u8> {
	let line_staticbuffer: *mut c_char = calloc(
		(BUFFER_SIZE + 1) as c_ulong,
		::core::mem::size_of::<c_char>() as c_ulong,
	) as *mut c_char;
	if line_staticbuffer.is_null() {
		return None;
	}
	let newline_pos = strchr(static_buffer as *const c_char, '\n' as i32) as *const u8;
	let len = (newline_pos.offset_from(static_buffer) as libc::c_long + 1_i64) as usize;
	line_staticbuffer.copy_from_nonoverlapping(static_buffer as *mut i8, len);
	shift_static_buffer(static_buffer);
	Some(line_staticbuffer as *mut u8)
}
#[allow(unsafe_op_in_unsafe_fn)]
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
			read_buffer((static_buffer[fd]).as_mut_ptr())
		} else {
			read_newln(
				fd as RawFd,
				&mut count,
				(static_buffer[fd]).as_mut_ptr(),
				&mut return_line,
			)
		},
	)
}
