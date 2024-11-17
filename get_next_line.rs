#![allow(static_mut_refs)]
use std::ptr::copy_nonoverlapping;

const BUF_SIZE: i32 = 16;
const BUF_SIZE_ONE: usize = BUF_SIZE as usize + 1;

use ::libc;
use libft_rs::{
	ft_calloc::ft_calloc, ft_memcpy::ft_memcpy, ft_memset::ft_memset, ft_strlcpy::ft_strlcpy,
};
extern "C" {
	fn free(_: *mut libc::c_void);
	fn read(__fd: libc::c_int, __buf: *mut libc::c_void, __nbytes: size_t) -> ssize_t;
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
}

///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// Takes a string, a character, and a maximum length.
/// Returns the index of the character in the string.
/// If the character is not found, returns the maximum length.
#[no_mangle]
pub unsafe extern "C" fn index_of(
	mut str: *mut libc::c_char,
	mut max_len: libc::c_int,
) -> libc::c_int {
	let mut i: libc::c_int = 0;
	while i < max_len
		&& *str.offset(i as isize) as libc::c_int != '\n' as libc::c_int
		&& *str.offset(i as isize) as libc::c_int != '\0' as i32
	{
		i += 1;
	}
	i
}

pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;
unsafe extern "C" fn check_free(
	mut buf: *mut libc::c_char,
	mut buf_idx: libc::c_int,
	mut line: *mut libc::c_char,
	mut is_buf: bool,
) -> *mut libc::c_char {
	let mut buf_nl_idx: libc::c_int = 0;
	let mut ret: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
	let mut gnl_idx: libc::c_int = 0;
	if line.is_null() {
		return std::ptr::null_mut::<libc::c_char>();
	}
	if is_buf {
		buf_nl_idx = index_of(buf, 2147483647 as libc::c_int);
		// ft_memcpy(
		// 	line as *mut libc::c_void,
		// 	buf as *const libc::c_void,
		// 	(buf_idx + 1 as libc::c_int) as size_t,
		// );
		copy_nonoverlapping(buf, line, (buf_idx + 1).try_into().unwrap());
		if *buf.offset(buf_nl_idx as isize) as libc::c_int != '\n' as i32 {
			*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
		} else {
			buf_nl_idx += 1;
		}
		// ft_memcpy(
		// 	buf as *mut libc::c_void,
		// 	buf.offset(buf_nl_idx as isize) as *const libc::c_void,
		// 	(BUF_SIZE - buf_nl_idx + 1 as libc::c_int) as size_t,
		// );
		copy_nonoverlapping(
			buf.offset(buf_nl_idx as isize) as *const libc::c_void,
			buf as *mut libc::c_void,
			(BUF_SIZE - buf_nl_idx + 1 as libc::c_int)
				.try_into()
				.unwrap(),
		);
	}
	gnl_idx = index_of(line, 2147483647 as libc::c_int);
	if *line.offset(gnl_idx as isize) as libc::c_int == '\n' as i32 {
		gnl_idx += 1;
	}
	ret = ft_calloc(
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		(gnl_idx + 1 as libc::c_int) as size_t,
	) as *mut libc::c_char;
	if ret.is_null() {
		free(line as *mut libc::c_void);
		return std::ptr::null_mut::<libc::c_void>() as *mut libc::c_char;
	}
	// ft_memcpy(
	// 	ret as *mut libc::c_void,
	// 	line as *const libc::c_void,
	// 	gnl_idx as size_t,
	// );
	copy_nonoverlapping(line, ret, gnl_idx.try_into().unwrap());
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
pub unsafe extern "C" fn get_next_line(mut fd: libc::c_int) -> *mut libc::c_char {
	let mut line: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
	static mut buf: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	let mut buf_idx: libc::c_int = 0;
	if fd < 0 as libc::c_int || BUF_SIZE < 1 as libc::c_int {
		return std::ptr::null_mut::<libc::c_char>();
	}
	line = std::ptr::null_mut::<libc::c_char>();
	buf_idx = -(1 as libc::c_int);
	loop {
		buf_idx += 1;
		if !(buf_idx < BUF_SIZE && buf[buf_idx as usize] as libc::c_int != 0) {
			break;
		}
		if buf[buf_idx as usize] as libc::c_int == '\n' as i32 {
			line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				(BUF_SIZE + 1 as libc::c_int) as size_t,
			) as *mut libc::c_char;
			if line.is_null() {
				return std::ptr::null_mut::<libc::c_char>();
			}
			return check_free(buf.as_mut_ptr(), buf_idx, line, 1 as libc::c_int != 0);
		}
	}
	if buf[buf_idx as usize] as libc::c_int != '\n' as i32 {
		read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
	}
	check_free(buf.as_mut_ptr(), buf_idx, line, 0 as libc::c_int != 0)
}
#[inline]
unsafe extern "C" fn iter_line(
	mut line: *mut *mut libc::c_char,
	mut buf: *mut libc::c_char,
	mut tmp: *mut libc::c_char,
	mut buf_idx: libc::c_int,
) -> bool {
	let mut buf_nl_idx: libc::c_int = 0;
	*line = ft_calloc(
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		(buf_idx + 1 as libc::c_int) as size_t,
	) as *mut libc::c_char;
	if (*line).is_null() {
		return 0 as libc::c_int != 0;
	}
	ft_strlcpy(*line, buf, (buf_idx + 1 as libc::c_int) as size_t);
	ft_memcpy(
		buf as *mut libc::c_void,
		tmp as *const libc::c_void,
		BUF_SIZE as size_t,
	);
	buf_nl_idx = index_of(buf, BUF_SIZE + 1 as libc::c_int);
	if *buf.offset(buf_nl_idx as isize) as libc::c_int != '\n' as i32 {
		*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
	} else {
		buf_nl_idx += 1;
	}
	ft_memcpy(
		buf as *mut libc::c_void,
		buf.offset(buf_nl_idx as isize) as *const libc::c_void,
		(BUF_SIZE - buf_nl_idx + 1 as libc::c_int) as size_t,
	);
	1 as libc::c_int != 0
}
unsafe extern "C" fn read_line(
	mut buf: *mut libc::c_char,
	mut fd: libc::c_int,
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
	let mut tmp_nl_idx: libc::c_int = 0;
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
	tmp_nl_idx = index_of(tmp.as_mut_ptr(), BUF_SIZE);
	if (tmp[tmp_nl_idx as usize] as libc::c_int == '\n' as i32
		|| rd == 0 as libc::c_int && *buf_idx != 0 as libc::c_int)
		&& !iter_line(line, buf, tmp.as_mut_ptr(), *buf_idx)
	{
		return std::ptr::null_mut::<libc::c_char>();
	}
	if tmp[tmp_nl_idx as usize] as libc::c_int != '\n' as i32
		&& rd != 0 as libc::c_int
		&& (read_line(buf, fd, buf_idx, line)).is_null()
	{
		return std::ptr::null_mut::<libc::c_char>();
	}
	if rd > 0 as libc::c_int && *buf_idx != 0 as libc::c_int {
		*buf_idx -= BUF_SIZE;
		tmp_nl_idx = index_of(tmp.as_mut_ptr(), BUF_SIZE);
		// ft_memcpy(
		// 	(*line).offset(*buf_idx as isize) as *mut libc::c_void,
		// 	tmp.as_mut_ptr() as *const libc::c_void,
		// 	tmp_nl_idx as size_t,
		// );
		copy_nonoverlapping(
			tmp.as_mut_ptr(),
			(*line).offset(*buf_idx as isize),
			tmp_nl_idx.try_into().unwrap(),
		);
		if tmp[tmp_nl_idx as usize] as libc::c_int == '\n' as i32 {
			*(*line).offset((*buf_idx + tmp_nl_idx) as isize) = '\n' as i32 as libc::c_char;
		}
	}
	*line
}
