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

unsafe fn allocate_for_c(size: usize) -> *mut i8 {
	let layout = Layout::array::<i8>(size).unwrap();
	unsafe {
		let ptr = alloc(layout) as *mut i8;
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
pub unsafe extern "C" fn get_next_line(fd: RawFd) -> *mut i8 {
	unsafe {
		static mut buf: [i8; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
		if fd < 0 as libc::c_int || BUF_USIZE < 1 {
			return std::ptr::null_mut::<i8>();
		}
		let mut buf_idx = 0;
		while buf_idx < BUF_USIZE && libc::c_int::from(buf[buf_idx]) != 0 {
			if libc::c_int::from(buf[buf_idx]) == '\n' as i32 {
				let mut line_alloc = vec![0i8, BUF_SIZE_ONE as i8];
				let mut buf_nl_idx: usize = index_of(buf.as_ptr(), 2_147_483_647);
				std::ptr::copy_nonoverlapping(buf.as_ptr(), line_alloc.as_mut_ptr(), buf_idx + 1);
				if *buf.as_ptr().add(buf_nl_idx) == '\n' as i8 {
					buf_nl_idx += 1;
				} else {
					*buf.as_mut_ptr().add(buf_nl_idx) = 0;
				}
				buf.copy_within(buf_nl_idx.., 0);
				let mut gnl_idx: usize = index_of(line_alloc.as_ptr().cast(), 2_147_483_647);
				if *line_alloc.as_ptr().add(gnl_idx) == '\n' as i8 {
					gnl_idx += 1;
				}
				let ret = allocate_for_c(gnl_idx + 1);
				if ret.is_null() {
					return std::ptr::null_mut::<i8>();
				}
				std::ptr::copy_nonoverlapping(line_alloc.as_ptr(), ret, gnl_idx);
				return ret;
			}
			buf_idx += 1;
		}
		let mut line: *mut i8 = std::ptr::null_mut::<i8>();
		if libc::c_int::from(buf[buf_idx]) != '\n' as i32 {
			read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
		}
		{
			if line.is_null() {
				return std::ptr::null_mut::<i8>();
			}
			let mut gnl_idx: usize = index_of(line, 2_147_483_647);
			if libc::c_int::from(*line.add(gnl_idx)) == '\n' as i32 {
				gnl_idx += 1;
			}
			let ret = allocate_for_c(gnl_idx + 1);
			if ret.is_null() {
				free(line.cast::<libc::c_void>());
				return std::ptr::null_mut::<i8>();
			}
			std::ptr::copy_nonoverlapping(line, ret, gnl_idx);
			ret
		}
	}
}

unsafe extern "C" fn read_line(
	buf: *mut i8,
	fd: RawFd,
	buf_idx: &mut usize,
	line: &mut *mut i8,
) -> *mut i8 {
	let mut tmp: [i8; BUF_SIZE_ONE] = [0; BUF_SIZE_ONE];
	unsafe {
		let rd: ssize_t = read(
			fd,
			tmp.as_mut_ptr().cast::<libc::c_void>(),
			(BUF_USIZE as size_t).try_into().unwrap(),
		) as ssize_t;
		if rd == -1 {
			buf.write_bytes(0, BUF_USIZE);
			return buf.cast::<i8>();
		}
		if rd > 0 {
			*buf_idx += BUF_USIZE;
		}
		let mut tmp_nl_idx: usize = index_of(tmp.as_mut_ptr(), BUF_USIZE);
		if (tmp[tmp_nl_idx] == '\n' as i8 || rd == 0 && *buf_idx != 0)
			&& !{
				*line = allocate_for_c(*buf_idx + 1);
				std::ptr::copy_nonoverlapping(buf, *line, *buf_idx); // replaces strlcpy
				std::ptr::copy_nonoverlapping(
					tmp.as_mut_ptr() as *const libc::c_void,
					buf.cast::<libc::c_void>(),
					BUF_USIZE,
				);
				let mut buf_nl_idx: usize = index_of(buf, BUF_USIZE + 1);
				if *buf.add(buf_nl_idx) == '\n' as i8 {
					buf_nl_idx += 1;
				} else {
					*buf.add(buf_nl_idx) = 0;
				}
				std::ptr::copy(
					buf.add(buf_nl_idx) as *const libc::c_void,
					buf.cast::<libc::c_void>(),
					BUF_USIZE - buf_nl_idx + 1,
				);
				true
			} {
			return std::ptr::null_mut::<i8>();
		}
		if tmp[tmp_nl_idx] != '\n' as i8 && rd != 0 {
			if (read_line(buf, fd, buf_idx, line)).is_null() {
				return std::ptr::null_mut::<i8>();
			}
		}
		if rd > 0 && *buf_idx != 0 {
			*buf_idx -= BUF_USIZE;
			tmp_nl_idx = index_of(tmp.as_mut_ptr(), BUF_USIZE);
			std::ptr::copy_nonoverlapping(tmp.as_mut_ptr(), (*line).add(*buf_idx), tmp_nl_idx);
			if tmp[tmp_nl_idx] == '\n' as i8 {
				*(*line).add(*buf_idx + tmp_nl_idx) = '\n' as i8;
			}
		}
		*line
	}
}
