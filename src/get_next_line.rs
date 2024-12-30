#![allow(static_mut_refs)]

const BUF_USIZE: usize = 16;

use std::{ffi::CString, os::fd::RawFd};

use ::libc;
use libft_rs::ft_strlcpy::ft_strlcpy;
use nix::unistd::read;
fn index_of(str: &[u8], max_len: usize) -> usize {
	let mut i: usize = 0;
	while i < max_len && str[i] != b'\n' && str[i] != b'\0' {
		i += 1;
	}
	i
}

pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;
#[allow(unused_mut)]
unsafe fn check_free(
	mut buf: &mut [u8],
	buf_idx: usize,
	mut line: Option<Vec<u8>>,
	is_buf: bool,
) -> Option<CString> {
	if line.is_none() {
		return None;
	}
	let mut line_bytes = CString::from_vec_with_nul_unchecked(line.unwrap());
	let mut line = line_bytes.into_bytes_with_nul();
	if is_buf {
		let mut buf_nl_idx: usize = index_of(buf, 2_147_483_647);
		line.extend_from_slice(&buf[..buf_idx]);
		if buf[buf_nl_idx] == b'\n' {
			buf_nl_idx += 1;
		} else {
			buf[buf_nl_idx] = b'\0';
		}
		buf.copy_within(buf_nl_idx..BUF_USIZE - buf_nl_idx + 1, 0);
	}
	let mut gnl_idx: usize = index_of(&line, 2_147_483_647);
	if line[gnl_idx] == b'\n' {
		gnl_idx += 1;
	}
	// copy gnl_idx bytes from line to ret
	Some(CString::from_vec_with_nul_unchecked(
		line[..gnl_idx + 1].to_vec(),
	))
}

///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// Takes a file descriptor.
/// Returns the next line from the file descriptor.
/// If the file descriptor is invalid, returns a null pointer.
/// If the line is empty, returns a null pointer.
// original return type: *mut libc::c_char
// let mut line: *mut libc::c_char = None;
#[unsafe(no_mangle)]
pub unsafe fn get_next_line(fd: RawFd) -> Option<CString> {
	unsafe {
		static mut buf: [u8; BUF_USIZE] = [0; BUF_USIZE];
		if fd < 0 || BUF_USIZE < 1 {
			return None;
		}
		let mut buf_idx = 0;
		loop {
			if !(buf_idx < BUF_USIZE - 1 && buf[buf_idx] != b'\0') {
				break;
			}
			if buf[buf_idx] == b'\n' {
				let mut alloced_line = vec![0u8; BUF_USIZE];
				// BUF_USIZE is the maximum length of the line (withouth the null terminator)
				return check_free(&mut buf, buf_idx, Some(alloced_line), true);
			}
			buf_idx += 1;
		}
		let line = CString::new("").unwrap();
		if buf[buf_idx] != b'\n' {
			let line = read_line(&mut buf, fd, &mut buf_idx, line);
			if line.is_none() {
				return check_free(&mut buf, buf_idx, None, false);
			} else {
				return check_free(&mut buf, buf_idx, Some(line.unwrap().into()), false);
			}
		}
		return check_free(&mut buf, buf_idx, None, false);
	}
}

unsafe fn read_line(
	buf: &mut [u8],
	fd: RawFd,
	buf_idx: &mut usize,
	line: CString,
) -> Option<CString> {
	let mut tmp: [u8; BUF_USIZE] = [0; BUF_USIZE];
	unsafe {
		tmp.as_mut_ptr().write_bytes(0, BUF_USIZE);
		let rd_result = read(fd, tmp.as_mut_slice());
		if rd_result.is_err() {
			// *mut c_char buf.ptr::write_bytes(0, BUF_USIZE);
			/* Invokes memset on the specified pointer, setting count * size_of::<T>() bytes of memory starting at self to val.
			 */
			buf.as_mut_ptr().write_bytes(0, BUF_USIZE);
			return Some(CString::from_vec_with_nul(buf.into()).unwrap());
		}
		let rd = rd_result.unwrap();
		if rd > 0 {
			*buf_idx += BUF_USIZE;
		}
		let mut tmp_nl_idx = index_of(&tmp, BUF_USIZE);
		if (tmp[tmp_nl_idx] == b'\n' || rd == 0 && *buf_idx != 0)
			&& !{
				let alloced_bytes = *buf_idx + 1; // include null terminator
				let mut alloced_line = vec![0u8; alloced_bytes];
				// replace with something safer @todo
				if alloced_bytes < *buf_idx + tmp_nl_idx {
					alloced_line.resize(*buf_idx + tmp_nl_idx, 0);
				}
				alloced_line[*buf_idx..*buf_idx + tmp_nl_idx].copy_from_slice(&tmp[..tmp_nl_idx]);
				buf[..BUF_USIZE].copy_from_slice(&tmp[..BUF_USIZE]);
				let mut buf_nl_idx: usize = index_of(buf, BUF_USIZE + 1);
				if buf[buf_nl_idx] == b'\n' {
					buf_nl_idx += 1;
				} else {
					buf[buf_nl_idx] = b'\0';
				}
				buf.copy_within(buf_nl_idx.., 0);
				true
			} {
			return None;
		}
		if tmp[tmp_nl_idx] != b'\n'
			&& rd != 0
			&& (read_line(buf, fd, buf_idx, line.clone())).is_none()
		{
			return None;
		}
		if rd > 0 && *buf_idx != 0 {
			if line.is_empty() {
				if *buf_idx < BUF_USIZE {
					return Some(CString::from_vec_with_nul_unchecked(
						buf[..(*buf_idx)].to_vec(),
					));
				} else {
					return Some(CString::from_vec_with_nul_unchecked(buf.to_vec()));
				}
			}
			*buf_idx -= BUF_USIZE;
			tmp_nl_idx = index_of(&tmp, BUF_USIZE);
			let mut line = line.clone().into_bytes();
			line[*buf_idx..*buf_idx + tmp_nl_idx].copy_from_slice(&tmp[..tmp_nl_idx]);
			if tmp[tmp_nl_idx] == b'\n' {
				line[*buf_idx + tmp_nl_idx] = b'\n';
			}
			let line = CString::from_vec_with_nul(line.to_vec()).unwrap();
			return Some(line);
		}
		Some(line)
	}
}
