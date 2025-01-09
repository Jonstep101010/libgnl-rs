#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]

use std::{
	clone::CloneToUninit,
	ffi::{c_char, c_ulong},
	mem::ManuallyDrop,
	os::fd::RawFd,
};
const ALLOC_SIZE: c_ulong = core::mem::size_of::<u8>() as c_ulong;
include!(concat!(env!("OUT_DIR"), "/buffer_size.rs"));

fn nl_position(buffer: &[u8]) -> Option<usize> {
	buffer.iter().position(|c| *c == b'\n')
}

unsafe extern "C" {
	fn malloc(_: c_ulong) -> *mut libc::c_void;
}

// vars depending on current stack frame:
// count += BUFFER_SIZE/ count + (BUFFER_SIZE * num_calls)
// read_buffer: populated with each call, copied into allocation (nl/EOF) - could push one for each loop iteration
// read_result -> new one for each call
// return_line -> None until populated

///
/// allocates on the heap only once EOL/EOF found
/// uses recursion otherwise
/// copies bytes once walking back up the stack
///
/// at the beginning:
/// ```no-run
/// assert!(!static_buffer.contains(&b'\n'));
/// assert!(return_line.is_none());
/// ```
fn read_newln(
	fd: RawFd,
	count: usize,
	static_buffer: &mut [u8; BUFFER_SIZE],
	mut return_line: Option<ManuallyDrop<Vec<u8>>>,
) -> Option<ManuallyDrop<Vec<u8>>> {
	let mut read_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	let read_result = nix::unistd::read(fd, read_buffer.as_mut_slice());
	if read_result.is_err() || read_result.unwrap() == 0 && count == 0 {
		static_buffer.fill(b'\0');
		None
	} else if read_result.unwrap() != 0 {
		/* read buffer has data */
		if let Some(newline_pos) = nl_position(&read_buffer[..]) {
			let mut alloc_nln = vec![0; count + BUFFER_SIZE + 1];
			// if there is non-zero data, we want it at the beginning of the line
			static_buffer.as_slice().clone_into(&mut alloc_nln);
			unsafe {
				// copy remainder of line into static_buffer, overwrite non-overwritten contents after copied
				read_buffer[newline_pos + 1..].clone_to_uninit(static_buffer.as_mut_ptr());
				static_buffer[(BUFFER_SIZE - (newline_pos + 1))..].fill(b'\0');
				read_buffer
					.as_ptr()
					.copy_to_nonoverlapping(alloc_nln.as_mut_ptr().add(count), newline_pos + 1);
			}
			Some(ManuallyDrop::new(alloc_nln))
		} else
		/* there is a remainder for the line */
		{
			return_line = read_newln(fd, count + BUFFER_SIZE, static_buffer, return_line);
			unsafe {
				read_buffer.as_ptr().copy_to_nonoverlapping(
					return_line
						.as_mut()
						.expect("recursive call to always return some")
						.as_mut_ptr()
						.add(count),
					BUFFER_SIZE,
				);
			}
			return_line
		}
	} else
	/* EOF reached (static contains data other than newline) */
	{
		assert_ne!(0, count, "EOF has to be reached with something read");
		let mut alloc_nul = vec![0; count + 1];
		static_buffer.as_slice().clone_into(&mut alloc_nul);
		// clean up since we're done with this fd
		static_buffer.fill(b'\0');
		Some(ManuallyDrop::new(alloc_nul))
	}
}

///
/// read a line from a buffer into heap memory and return a pointer to the heap memory
///
/// this will never be called if the buffer is empty:
/// ```no-run
/// assert!(!&static_buffer.starts_with(&[0; BUFFER_SIZE + 1]));
/// assert_eq!(static_buffer[count], b'\n');
/// ```
unsafe fn read_buffer(static_buffer: &mut [u8; BUFFER_SIZE], count: usize) -> *mut c_char {
	let copy_return_line = malloc((count + 2) as c_ulong * ALLOC_SIZE).cast::<u8>();
	if !copy_return_line.is_null() {
		static_buffer
			.as_ptr()
			.copy_to_nonoverlapping(copy_return_line, count + 1);
		*copy_return_line.add(count + 1) = b'\0';
	}
	static_buffer.copy_within(count + 1.., 0);
	static_buffer[(BUFFER_SIZE - count)..].fill(b'\0');
	copy_return_line.cast::<c_char>()
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
///
/// # Panics
/// this function should never panic. something has to go horribly wrong for the buffer to be fully traversed
#[allow(clippy::cast_sign_loss)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_next_line(fd: RawFd) -> *mut c_char {
	static mut static_buffer: [[u8; BUFFER_SIZE]; 10240] = [[0; BUFFER_SIZE]; 10240];
	if BUFFER_SIZE < 1 || !(0..=10240).contains(&fd) {
		return std::ptr::null_mut::<c_char>();
	}
	for (count, elem) in static_buffer[fd as usize].iter().enumerate() {
		if elem == &b'\n' {
			return read_buffer(&mut (static_buffer[fd as usize]), count);
		}
		if elem == &b'\0' {
			return match read_newln(fd, count, &mut (static_buffer[fd as usize]), Option::None) {
				Some(mut mandrop_line) => {
					let cstr_line = std::ffi::CStr::from_ptr(mandrop_line.as_ptr().cast::<i8>());
					let copy_return_line =
						malloc((cstr_line.count_bytes() + 1) as c_ulong * ALLOC_SIZE).cast::<u8>();
					if !copy_return_line.is_null() {
						cstr_line.clone_to_uninit(copy_return_line);
					}
					ManuallyDrop::drop(&mut mandrop_line);
					copy_return_line.cast::<c_char>()
				}
				None => std::ptr::null_mut::<c_char>(),
			};
		}
	}
	unreachable!("the loop should always return!")
}
