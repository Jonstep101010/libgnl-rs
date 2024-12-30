#![allow(static_mut_refs)]

const BUF_USIZE: usize = 16;
const BUF_SIZE_ONE: usize = BUF_USIZE + 1;
use ::libc;
use libc::{free, read};
use std::{
	alloc::{Layout, alloc},
	ffi::CStr,
	os::{fd::RawFd, raw::c_char},
};

// use nix::unistd::read;
fn index_of(str: *const c_char, max_len: usize) -> usize {
	let str = unsafe {
		let cstring = CStr::from_ptr(str as *mut i8);
		cstring.to_bytes_with_nul()
	};
	let mut i: usize = 0;
	while i < max_len && str[i] != b'\n' && str[i] != b'\0' {
		i += 1;
	}
	i
}

unsafe fn allocate_for_c(size: usize) -> *mut libc::c_char {
	let layout = Layout::array::<libc::c_char>(size).unwrap();
	unsafe {
		let ptr = alloc(layout) as *mut libc::c_char;
		if ptr.is_null() {
			std::alloc::handle_alloc_error(layout);
		}
		ptr
	}
}

pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;

///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// Takes a file descriptor.
/// Returns the next line from the file descriptor.
/// If the file descriptor is invalid, returns a null pointer.
/// If the line is empty, returns a null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(fd: RawFd) -> *mut libc::c_char {
	unsafe {
		static mut buf: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
		if fd < 0 as libc::c_int || BUF_USIZE < 1 {
			return std::ptr::null_mut::<libc::c_char>();
		}
		let mut buf_idx = 0;
		while buf_idx < BUF_USIZE && libc::c_int::from(buf[buf_idx]) != 0 {
			if libc::c_int::from(buf[buf_idx]) == '\n' as i32 {
				let mut line_alloc = vec![0u8, BUF_SIZE_ONE as u8];
				let mut buf_nl_idx: usize = index_of(buf.as_ptr(), 2_147_483_647);
				std::ptr::copy_nonoverlapping(
					buf.as_ptr(),
					line_alloc.as_mut_ptr() as *mut i8,
					buf_idx + 1,
				);
				if libc::c_int::from(*buf.as_ptr().add(buf_nl_idx)) == '\n' as i32 {
					buf_nl_idx += 1;
				} else {
					*buf.as_mut_ptr().add(buf_nl_idx) =
						libc::c_char::try_from(0 as libc::c_int).unwrap();
				}
				buf.copy_within(buf_nl_idx.., 0);
				let mut gnl_idx: usize = index_of(line_alloc.as_ptr().cast(), 2_147_483_647);
				if *line_alloc.as_ptr().add(gnl_idx) == b'\n' {
					gnl_idx += 1;
				}
				let ret = allocate_for_c(gnl_idx + 1);
				if ret.is_null() {
					return std::ptr::null_mut::<libc::c_char>();
				}
				std::ptr::copy_nonoverlapping(line_alloc.as_ptr(), ret as *mut u8, gnl_idx);
				return ret;
			}
			buf_idx += 1;
		}
		let mut line: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
		if libc::c_int::from(buf[buf_idx]) != '\n' as i32 {
			read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
		}
		{
			if line.is_null() {
				return std::ptr::null_mut::<libc::c_char>();
			}
			let mut gnl_idx: usize = index_of(line, 2_147_483_647);
			if libc::c_int::from(*line.add(gnl_idx)) == '\n' as i32 {
				gnl_idx += 1;
			}
			let ret = allocate_for_c(gnl_idx + 1);
			if ret.is_null() {
				free(line.cast::<libc::c_void>());
				return std::ptr::null_mut::<libc::c_char>();
			}
			std::ptr::copy_nonoverlapping(line, ret, gnl_idx);
			ret
		}
	}
}

unsafe extern "C" fn read_line(
	buf: *mut libc::c_char,
	fd: RawFd,
	buf_idx: *mut usize,
	line: *mut *mut libc::c_char,
) -> *mut libc::c_char {
	let mut tmp: [libc::c_char; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	unsafe {
		let rd: ssize_t = read(
			fd,
			tmp.as_mut_ptr().cast::<libc::c_void>(),
			(BUF_USIZE as size_t).try_into().unwrap(),
		) as ssize_t;
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
				*line = allocate_for_c(*buf_idx + 1);
				std::ptr::copy_nonoverlapping(buf, *line, *buf_idx); // replaces strlcpy
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
}
