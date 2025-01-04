#![allow(
	dead_code,
	mutable_transmutes,
	non_camel_case_types,
	non_snake_case,
	non_upper_case_globals,
	unused_assignments,
	unused_mut
)]
#![allow(static_mut_refs)]

use std::ptr::drop_in_place;
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

unsafe extern "C" {
	pub type __sFILEX;
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
	fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
	// fn free(_: *mut libc::c_void);
	// fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
	// fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
	// -> *mut libc::c_void;
	// fn memset(_: *mut libc::c_void, _: core::ffi::c_int, _: libc::c_ulong) -> *mut libc::c_void;
	fn strchr(_: *const core::ffi::c_char, _: core::ffi::c_int) -> *mut core::ffi::c_char;
	// fn strlen(_: *const core::ffi::c_char) -> libc::c_ulong;
	// fn bzero(_: *mut libc::c_void, _: libc::c_ulong);
	// only used if building with main
	fn getcwd(_: *mut core::ffi::c_char, _: size_t) -> *mut core::ffi::c_char;
	fn read(_: core::ffi::c_int, _: *mut libc::c_void, _: size_t) -> ssize_t;
	fn open(_: *const core::ffi::c_char, _: core::ffi::c_int, _: ...) -> core::ffi::c_int;
	static mut __stderrp: *mut FILE;
	fn fprintf(_: *mut FILE, _: *const core::ffi::c_char, _: ...) -> core::ffi::c_int;
}
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_ssize_t = libc::c_long;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type ssize_t = __darwin_ssize_t;
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: core::ffi::c_int,
	pub _w: core::ffi::c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: core::ffi::c_int,
	pub _cookie: *mut libc::c_void,
	pub _close: Option<unsafe extern "C" fn(*mut libc::c_void) -> core::ffi::c_int>,
	pub _read: Option<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*mut core::ffi::c_char,
			core::ffi::c_int,
		) -> core::ffi::c_int,
	>,
	pub _seek: Option<unsafe extern "C" fn(*mut libc::c_void, fpos_t, core::ffi::c_int) -> fpos_t>,
	pub _write: Option<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const core::ffi::c_char,
			core::ffi::c_int,
		) -> core::ffi::c_int,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: core::ffi::c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: core::ffi::c_int,
	pub _offset: fpos_t,
}
pub type FILE = __sFILE;
#[unsafe(no_mangle)]
unsafe extern "C" fn shift_static_buffer(static_buffer: *mut core::ffi::c_char) {
	unsafe {
		let mut newline_pos: *const core::ffi::c_char =
			strchr(static_buffer as *const core::ffi::c_char, '\n' as i32);
		if newline_pos.is_null() {
			static_buffer.write_bytes(b'\0', BUFFER_SIZE);
		} else {
			// get index after '\n'
			let start = newline_pos.offset_from(static_buffer) + 1;
			let shift_len: usize = (BUFFER_SIZE + 1).wrapping_sub_signed(start);
			// shift contents from after '\n' to beginning
			std::ptr::copy(static_buffer.offset(start), static_buffer, shift_len);
			static_buffer
				.add(shift_len)
				.write_bytes(b'\0', (BUFFER_SIZE).wrapping_sub(shift_len));
		}
	}
}
#[unsafe(no_mangle)]
unsafe extern "C" fn terminated_line_copy(
	mut return_line: *mut core::ffi::c_char,
) -> *mut core::ffi::c_char {
	if return_line.is_null() {
		return std::ptr::null_mut::<core::ffi::c_char>();
	}
	unsafe {
		// let len = strlen(return_line);
		let len = std::ffi::CStr::from_ptr(return_line).count_bytes();
		let mut copy_return_line: *mut core::ffi::c_char =
			malloc((len + 1).wrapping_mul(::core::mem::size_of::<core::ffi::c_char>()) as size_t)
				as *mut core::ffi::c_char;
		if !copy_return_line.is_null() {
			std::ptr::copy_nonoverlapping(return_line, copy_return_line, len + 1);
		}
		// free(return_line as *mut libc::c_void);
		drop_in_place(return_line);
		copy_return_line
	}
}

///
/// returns: `return_line`
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn copy_into_return_line(
	count: &mut usize,
	return_line: *mut i8,
	temp_buffer: *const i8,
) -> *mut i8 {
	if *temp_buffer != '\0' as i8 {
		*count -= BUFFER_SIZE;
		let newline: *const core::ffi::c_char = strchr(temp_buffer, '\n' as i32);
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
		std::ptr::copy_nonoverlapping(temp_buffer, return_line.add(*count), len);
	}
	return_line
}
#[unsafe(no_mangle)]
unsafe extern "C" fn read_newln(
	fd: usize,
	mut count: &mut usize,
	mut static_buffer: *mut core::ffi::c_char,
	mut return_line: *mut *mut core::ffi::c_char,
) -> *mut core::ffi::c_char {
	let mut temp_buffer: [core::ffi::c_char; BUFFER_SIZE + 1] = [0; BUFFER_SIZE + 1];
	unsafe {
		let bytes_read: ssize_t = read(
			fd as core::ffi::c_int,
			temp_buffer.as_mut_ptr() as *mut libc::c_void,
			BUFFER_SIZE as size_t,
		);
		if bytes_read > 0 {
			*count += BUFFER_SIZE;
		} else if bytes_read < 0 || bytes_read == 0 && *count == 0 {
			// *return_line = std::ptr::null_mut::<core::ffi::c_char>();
			// we assign so we do not need to assign by deref if returning to caller immediately
			static_buffer.write_bytes(0, BUFFER_SIZE + 1);
			return std::ptr::null_mut::<core::ffi::c_char>();
		}
		let temp_buffer_c_str = std::ffi::CStr::from_ptr(temp_buffer.as_ptr());
		// let mut newline_pos: *const core::ffi::c_char = strchr(temp_buffer.as_mut_ptr(), '\n' as i32);
		let newline_pos = temp_buffer_c_str.bytes().find(|&c| c == b'\n');
		// if !newline_pos.is_null() || bytes_read == 0 && *count != 0 as core::ffi::c_int {
		if newline_pos.is_some() || bytes_read == 0 && *count != 0 {
			*return_line = calloc(
				(*count + 1) as libc::c_ulong,
				::core::mem::size_of::<core::ffi::c_char>() as libc::c_ulong,
			) as *mut core::ffi::c_char;
			if (*return_line).is_null() {
				return std::ptr::null_mut::<core::ffi::c_char>();
			}
			std::ptr::copy_nonoverlapping(
				static_buffer,
				*return_line,
				// strlen(static_buffer as *const core::ffi::c_char) as usize,
				std::ffi::CStr::from_ptr(static_buffer).count_bytes(),
			);
			std::ptr::copy_nonoverlapping(temp_buffer.as_ptr(), static_buffer, BUFFER_SIZE);
			shift_static_buffer(static_buffer);
		// } else if newline_pos.is_null() && bytes_read != 0 {
		} else if newline_pos.is_none() && bytes_read != 0 {
			*return_line = read_newln(fd, count, static_buffer, return_line);
		}
		copy_into_return_line(count, *return_line, temp_buffer.as_ptr())
	}
}

#[unsafe(no_mangle)]
unsafe extern "C" fn read_buffer(
	mut static_buffer: *mut core::ffi::c_char,
) -> *mut core::ffi::c_char {
	unsafe {
		let mut line_staticbuffer: *mut core::ffi::c_char = calloc(
			(BUFFER_SIZE + 1) as libc::c_ulong,
			::core::mem::size_of::<core::ffi::c_char>() as libc::c_ulong,
		) as *mut core::ffi::c_char;
		if line_staticbuffer.is_null() {
			return std::ptr::null_mut::<core::ffi::c_char>();
		}
		let mut newline_pos: *const core::ffi::c_char =
			strchr(static_buffer as *const core::ffi::c_char, '\n' as i32);
		let len: size_t =
			(newline_pos.offset_from(static_buffer) as libc::c_long + 1_i64) as size_t;
		std::ptr::copy_nonoverlapping(static_buffer, line_staticbuffer, len as usize);
		shift_static_buffer(static_buffer);
		line_staticbuffer
	}
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(fd: std::os::fd::RawFd) -> *mut core::ffi::c_char {
	if (BUFFER_SIZE as core::ffi::c_int) < 1 as core::ffi::c_int || !(0..=10240).contains(&fd) {
		return std::ptr::null_mut::<core::ffi::c_char>();
	}
	let fd: usize = fd as usize;
	unsafe {
		static mut static_buffer: [[core::ffi::c_char; BUFFER_SIZE + 1]; 10240] =
			[[0; BUFFER_SIZE + 1]; 10240];
		let mut count: usize = 0;
		while static_buffer[fd][count] as core::ffi::c_int != '\0' as i32
			&& static_buffer[fd][count] as core::ffi::c_int != '\n' as i32
		{
			count += 1;
		}
		let mut return_line: *mut core::ffi::c_char = std::ptr::null_mut::<core::ffi::c_char>();
		terminated_line_copy(
			if count <= BUFFER_SIZE && static_buffer[fd][count] as core::ffi::c_int == '\n' as i32 {
				read_buffer((static_buffer[fd]).as_mut_ptr())
			} else {
				read_newln(
					fd,
					&mut count,
					(static_buffer[fd]).as_mut_ptr(),
					&mut return_line,
				)
			},
		)
	}
}
unsafe fn main_0() -> core::ffi::c_int {
	unsafe {
		fprintf(
			__stderrp,
			b"%s\n\0" as *const u8 as *const core::ffi::c_char,
			getcwd(
				std::ptr::null_mut::<core::ffi::c_char>(),
				0 as core::ffi::c_int as size_t,
			),
		);
		let mut fd: core::ffi::c_int = open(
			b"./.clang-tidy\0" as *const u8 as *const core::ffi::c_char,
			0 as core::ffi::c_int,
		);
		let mut line: *mut core::ffi::c_char = get_next_line(fd);
		while !line.is_null() {
			fprintf(
				__stderrp,
				b"%s\n\0" as *const u8 as *const core::ffi::c_char,
				line,
			);
			drop_in_place(line);
			line = get_next_line(fd);
		}
		0
	}
}
pub fn main() {
	unsafe { ::std::process::exit(main_0() as i32) }
}
