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
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

unsafe extern "C" {
	pub type __sFILEX;
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
	fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
	fn free(_: *mut libc::c_void);
	fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
	fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
	-> *mut libc::c_void;
	fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
	fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
	fn bzero(_: *mut libc::c_void, _: libc::c_ulong);
	fn getcwd(_: *mut libc::c_char, _: size_t) -> *mut libc::c_char;
	fn read(_: libc::c_int, _: *mut libc::c_void, _: size_t) -> ssize_t;
	fn open(_: *const libc::c_char, _: libc::c_int, _: ...) -> libc::c_int;
	static mut __stderrp: *mut FILE;
	fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
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
	pub _size: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: libc::c_int,
	pub _w: libc::c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: libc::c_int,
	pub _cookie: *mut libc::c_void,
	pub _close: Option<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
	pub _read: Option<
		unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_char, libc::c_int) -> libc::c_int,
	>,
	pub _seek: Option<unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t>,
	pub _write: Option<
		unsafe extern "C" fn(*mut libc::c_void, *const libc::c_char, libc::c_int) -> libc::c_int,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: libc::c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: libc::c_int,
	pub _offset: fpos_t,
}
pub type FILE = __sFILE;
#[unsafe(no_mangle)]
unsafe extern "C" fn shift_static_buffer(mut static_buffer: *mut libc::c_char) {
	unsafe {
		let mut newline_pos: *const libc::c_char =
			strchr(static_buffer as *const libc::c_char, '\n' as i32);
		if newline_pos.is_null() {
			memset(
				static_buffer as *mut libc::c_void,
				'\0' as i32,
				BUFFER_SIZE as libc::c_int as libc::c_ulong,
			);
			return;
		}
		let start: size_t = (newline_pos.offset_from(static_buffer) as libc::c_long
			+ 1 as libc::c_int as libc::c_long) as size_t;
		let shift_len: size_t =
			((BUFFER_SIZE as libc::c_int + 1 as libc::c_int) as size_t).wrapping_sub(start);
		memmove(
			static_buffer as *mut libc::c_void,
			static_buffer.offset(start as isize) as *const libc::c_void,
			shift_len,
		);
		memset(
			static_buffer.offset(shift_len as isize) as *mut libc::c_void,
			0 as libc::c_int,
			(BUFFER_SIZE as libc::c_int as size_t).wrapping_sub(shift_len),
		);
	}
}
#[unsafe(no_mangle)]
unsafe extern "C" fn terminated_line_copy(mut return_line: *mut libc::c_char) -> *mut libc::c_char {
	if return_line.is_null() {
		return 0 as *mut libc::c_char;
	}
	unsafe {
		let len: size_t = strlen(return_line);
		let mut copy_return_line: *mut libc::c_char = malloc(
			len.wrapping_add(1 as libc::c_int as size_t)
				.wrapping_mul(::core::mem::size_of::<libc::c_char>() as libc::c_ulong),
		) as *mut libc::c_char;
		if copy_return_line.is_null() {
			free(return_line as *mut libc::c_void);
			return 0 as *mut libc::c_char;
		}
		bzero(
			copy_return_line as *mut libc::c_void,
			len.wrapping_add(1 as libc::c_int as size_t),
		);
		memcpy(
			copy_return_line as *mut libc::c_void,
			return_line as *const libc::c_void,
			len,
		);
		free(return_line as *mut libc::c_void);
		return copy_return_line;
	}
}
#[unsafe(no_mangle)]
unsafe extern "C" fn read_newln(
	fd: libc::c_int,
	mut count: *mut libc::c_int,
	mut static_buffer: *mut libc::c_char,
	mut return_line: *mut *mut libc::c_char,
) -> *mut libc::c_char {
	let mut temp_buffer: [libc::c_char; BUFFER_SIZE + 1] = [0; BUFFER_SIZE + 1];
	unsafe {
		let bytes_read: ssize_t = read(
			fd,
			temp_buffer.as_mut_ptr() as *mut libc::c_void,
			BUFFER_SIZE as libc::c_int as size_t,
		);
		if bytes_read > 0 as libc::c_int as ssize_t {
			*count += BUFFER_SIZE as libc::c_int;
		} else if bytes_read < 0 as libc::c_int as ssize_t
			|| bytes_read == 0 as libc::c_int as ssize_t && *count == 0 as libc::c_int
		{
			*return_line = 0 as *mut libc::c_char;
			bzero(
				static_buffer as *mut libc::c_void,
				(BUFFER_SIZE as libc::c_int + 1 as libc::c_int) as libc::c_ulong,
			);
			return 0 as *mut libc::c_char;
		}
		let mut newline_pos: *const libc::c_char = strchr(temp_buffer.as_mut_ptr(), '\n' as i32);
		if !newline_pos.is_null()
			|| bytes_read == 0 as libc::c_int as ssize_t && *count != 0 as libc::c_int
		{
			*return_line = calloc(
				(*count + 1 as libc::c_int) as libc::c_ulong,
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
			) as *mut libc::c_char;
			if (*return_line).is_null() {
				return 0 as *mut libc::c_char;
			}
			memcpy(
				*return_line as *mut libc::c_void,
				static_buffer as *const libc::c_void,
				strlen(static_buffer as *const libc::c_char),
			);
			memcpy(
				static_buffer as *mut libc::c_void,
				temp_buffer.as_mut_ptr() as *const libc::c_void,
				BUFFER_SIZE as libc::c_int as libc::c_ulong,
			);
			shift_static_buffer(static_buffer);
		} else if newline_pos.is_null() && bytes_read != 0 as libc::c_int as ssize_t {
			*return_line = read_newln(fd, count, static_buffer, return_line);
		}
		if *temp_buffer.as_mut_ptr() != 0 {
			*count -= BUFFER_SIZE as libc::c_int;
			let mut newline: *const libc::c_char = strchr(temp_buffer.as_mut_ptr(), '\n' as i32);
			if !newline.is_null() {
				let len: libc::c_int = (if (newline.offset_from(temp_buffer.as_mut_ptr())
					as libc::c_long)
					< BUFFER_SIZE as libc::c_int as libc::c_long
				{
					newline.offset_from(temp_buffer.as_mut_ptr()) as libc::c_long
				} else {
					BUFFER_SIZE as libc::c_int as libc::c_long
				}) as libc::c_int;
				memcpy(
					(*return_line).offset(*count as isize) as *mut libc::c_void,
					temp_buffer.as_mut_ptr() as *const libc::c_void,
					(len + 1 as libc::c_int) as libc::c_ulong,
				);
			} else {
				memcpy(
					(*return_line).offset(*count as isize) as *mut libc::c_void,
					temp_buffer.as_mut_ptr() as *const libc::c_void,
					BUFFER_SIZE as libc::c_int as libc::c_ulong,
				);
			}
		}
		return *return_line;
	}
}
#[unsafe(no_mangle)]
unsafe extern "C" fn read_buffer(mut static_buffer: *mut libc::c_char) -> *mut libc::c_char {
	unsafe {
		let mut line_staticbuffer: *mut libc::c_char = calloc(
			(BUFFER_SIZE + 1) as libc::c_ulong,
			::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		) as *mut libc::c_char;
		if line_staticbuffer.is_null() {
			return 0 as *mut libc::c_char;
		}
		let mut newline_pos: *const libc::c_char =
			strchr(static_buffer as *const libc::c_char, '\n' as i32);
		let len: size_t = (newline_pos.offset_from(static_buffer) as libc::c_long
			+ 1 as libc::c_int as libc::c_long) as size_t;
		memcpy(
			line_staticbuffer as *mut libc::c_void,
			static_buffer as *const libc::c_void,
			len,
		);
		shift_static_buffer(static_buffer);
		return line_staticbuffer;
	}
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(mut fd: libc::c_int) -> *mut libc::c_char {
	if (BUFFER_SIZE as libc::c_int) < 1 as libc::c_int
		|| fd > 10240 as libc::c_int
		|| fd < 0 as libc::c_int
	{
		return 0 as *mut libc::c_char;
	}
	unsafe {
		static mut static_buffer: [[libc::c_char; BUFFER_SIZE + 1]; 10240] =
			[[0; BUFFER_SIZE + 1]; 10240];
		let mut count: libc::c_int = 0 as libc::c_int;
		while static_buffer[fd as usize][count as usize] as libc::c_int != '\0' as i32
			&& static_buffer[fd as usize][count as usize] as libc::c_int != '\n' as i32
		{
			count += 1;
		}
		let mut return_line: *mut libc::c_char = 0 as *mut libc::c_char;
		if count <= BUFFER_SIZE as libc::c_int
			&& static_buffer[fd as usize][count as usize] as libc::c_int == '\n' as i32
		{
			return terminated_line_copy(read_buffer((static_buffer[fd as usize]).as_mut_ptr()));
		}
		return terminated_line_copy(read_newln(
			fd,
			&mut count,
			(static_buffer[fd as usize]).as_mut_ptr(),
			&mut return_line,
		));
	}
}
unsafe fn main_0() -> libc::c_int {
	fprintf(
		__stderrp,
		b"%s\n\0" as *const u8 as *const libc::c_char,
		getcwd(0 as *mut libc::c_char, 0 as libc::c_int as size_t),
	);
	let mut fd: libc::c_int = open(
		b"./.clang-tidy\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
	);
	let mut line: *mut libc::c_char = get_next_line(fd);
	while !line.is_null() {
		fprintf(
			__stderrp,
			b"%s\n\0" as *const u8 as *const libc::c_char,
			line,
		);
		free(line as *mut libc::c_void);
		line = get_next_line(fd);
	}
	return 0;
}
pub fn main() {
	unsafe { ::std::process::exit(main_0() as i32) }
}
