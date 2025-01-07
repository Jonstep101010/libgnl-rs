#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]

use std::{
	clone::CloneToUninit,
	ffi::{c_char, c_int, c_ulong},
	os::fd::RawFd,
	ptr::slice_from_raw_parts_mut,
};
const ALLOC_SIZE: c_ulong = core::mem::size_of::<u8>() as c_ulong;
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

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
	// if call_number == 0 {

	// }
	let read_result = nix::unistd::read(fd, read_buffer.as_mut_slice());
	#[cfg(debug_assertions)]
	{
		if call_number == 0 {
			eprintln!(
				"new read cycle\n[{:?}]",
				std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
			);
		}
		match read_result {
			Err(_) => {
				eprintln!("read failed!");
				match return_line {
					Some(line) => {
						dbg!(std::ffi::CStr::from_ptr(*line as *const i8));
						dbg!(std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8));
					}
					None => {
						eprintln!("return_line is None");
						dbg!(
							std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
						);
						dbg!(std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8));
					}
				}
			}
			Ok(_) => {
				eprintln!(
					"{call_number}: read into buffer: {:?}",
					std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8)
				);
			}
		}
	}
	if read_result.is_err() || read_result.unwrap() == 0 && *count == 0 {
		static_buffer.fill(b'\0');
		#[cfg(debug_assertions)]
		{
			assert!(call_number == 0);
			eprintln!("-- finished cycle --\n");
		}
		return None;
	}
	match read_result.unwrap() {
		0 if *count != 0 => {
			// EOF reached (other case already handled)
			*return_line = Some({
				let mut alloc = vec![0; *count + 1];
				static_buffer.as_slice().clone_into(&mut alloc);
				let ptr = alloc.as_mut_ptr();
				#[cfg(debug_assertions)]
				{
					eprint!("{call_number}: ");
					dbg!(*count);
					eprintln!(
						"{}: EOF allocated (line , read, static): {:?},{:?},{:?}",
						call_number,
						return_line.map_or("None", |line| {
							std::ffi::CStr::from_ptr(line as *const i8)
								.to_str()
								.unwrap()
						}),
						std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8),
						std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
					);
				}
				std::mem::forget(alloc);
				ptr
			});
			static_buffer.fill(b'\0');
		}
		_ => {
			// read buffer has data
			*count += BUFFER_SIZE;
			#[cfg(debug_assertions)]
			eprintln!("{}: count++", call_number);
			if let Some(newline_pos) = read_buffer.as_slice().iter().position(|&c| c == b'\n') {
				*return_line = Some({
					let mut alloc = vec![0; *count + 1];
					// put beginning of line into heap memory
					static_buffer.as_slice().clone_into(&mut alloc);
					let ptr = alloc.as_mut_ptr();
					#[cfg(debug_assertions)]
					{
						eprint!("{call_number}: ");
						dbg!(*count);
						eprintln!(
							"{}: allocating line (line , read, static): {:?},{:?},{:?}",
							call_number,
							return_line.map_or("None", |line| {
								std::ffi::CStr::from_ptr(line as *const i8)
									.to_str()
									.unwrap()
							}),
							std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8),
							std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
						);
					}
					std::mem::forget(alloc);
					// only if the read buffer has data & has a newline
					static_buffer[..BUFFER_SIZE].copy_from_slice(&read_buffer[..]);
					ptr
				});
				static_buffer.copy_within(newline_pos + 1.., 0);
				static_buffer[(BUFFER_SIZE - newline_pos)..].fill(b'\0');
			} else {
				*return_line = read_newln(fd, count, static_buffer, return_line, call_number + 1);
				#[cfg(debug_assertions)]
				{
					eprint!("{call_number}: ");
					dbg!(*count);
					eprintln!(
						"{}: return from recursive (line , read, static): {:?},{:?},{:?}",
						call_number,
						return_line.map_or("None", |line| {
							std::ffi::CStr::from_ptr(line as *const i8)
								.to_str()
								.unwrap()
						}),
						std::ffi::CStr::from_ptr(read_buffer.as_ptr() as *const i8),
						std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
					);
				}
			}
			if read_buffer.as_slice()[0] != b'\0' {
				// non-empty read buffer
				#[cfg(test)]
				{
					eprintln!(
						"non-empty read (read, line): {:?},{:?}",
						std::ffi::CStr::from_bytes_with_nul_unchecked(read_buffer.as_slice()),
						std::ffi::CStr::from_ptr(return_line.unwrap() as *const i8)
					);
				}
				let cpy_from_read = &read_buffer.as_slice()[..match read_buffer
					.as_slice()
					.iter()
					.position(|&c| c == b'\n')
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
				#[cfg(debug_assertions)]
				dbg!(*count, cpy_from_read.len());
				#[cfg(debug_assertions)]
				eprintln!(
					"{}: count-- copying from read buffer: {:?}",
					call_number,
					std::ffi::CStr::from_bytes_with_nul_unchecked(cpy_from_read)
				);
				let slice_ret =
					slice_from_raw_parts_mut(return_line.unwrap(), *count + cpy_from_read.len());
				(*slice_ret)[*count..].copy_from_slice(cpy_from_read);
				#[cfg(debug_assertions)]
				eprintln!(
					"{}: return_line w/ copy: {:?}",
					call_number,
					std::ffi::CStr::from_bytes_with_nul_unchecked(slice_ret.as_ref_unchecked())
				);
				*return_line = Some(slice_ret.as_mut_ptr());
			}
		}
	}
	#[cfg(debug_assertions)]
	{
		eprintln!(
			"{}: returning with (line, _, static): {:?},_,{:?}",
			call_number,
			std::ffi::CStr::from_ptr(return_line.unwrap() as *const i8),
			std::ffi::CStr::from_bytes_until_nul(static_buffer.as_slice()).unwrap()
		);
		if call_number == 0 {
			eprintln!("-- finished cycle --\n");
			eprintln!();
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
