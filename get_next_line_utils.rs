use ::libc;
extern "C" {
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
}
pub type size_t = libc::c_ulong;
#[no_mangle]
pub unsafe extern "C" fn ft_memcpy(
	mut dst: *mut libc::c_void,
	mut src: *const libc::c_void,
	mut n: size_t,
) -> *mut libc::c_void {
	let mut d: *mut libc::c_char = 0 as *mut libc::c_char;
	let mut s: *const libc::c_char = src as *const libc::c_char;
	d = dst as *mut libc::c_char;
	if d.is_null() && s.is_null() {
		return 0 as *mut libc::c_void;
	}
	loop {
		let fresh0 = n;
		n = n.wrapping_sub(1);
		if !(fresh0 != 0 && (!d.is_null() || !s.is_null())) {
			break;
		}
		let fresh1 = s;
		s = s.offset(1);
		let fresh2 = d;
		d = d.offset(1);
		*fresh2 = *fresh1;
	}
	return dst;
}
#[no_mangle]
pub unsafe extern "C" fn ft_memset(
	mut str: *mut libc::c_void,
	mut c: libc::c_int,
	mut n: size_t,
) -> *mut libc::c_void {
	let mut i: size_t = 0;
	let mut ptr: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
	ptr = str as *mut libc::c_uchar;
	i = 0 as libc::c_int as size_t;
	while i < n {
		let fresh3 = i;
		i = i.wrapping_add(1);
		*ptr.offset(fresh3 as isize) = c as libc::c_uchar;
	}
	return str;
}
#[no_mangle]
pub unsafe extern "C" fn ft_calloc(mut nitems: size_t, mut size: size_t) -> *mut libc::c_void {
	let mut ptr: *mut libc::c_void = 0 as *mut libc::c_void;
	if nitems != 0 && nitems.wrapping_mul(size).wrapping_div(nitems) != size {
		return 0 as *mut libc::c_void;
	}
	ptr = malloc(nitems.wrapping_mul(size));
	if ptr.is_null() {
		return 0 as *mut libc::c_void;
	}
	return ft_memset(ptr, 0 as libc::c_int, nitems.wrapping_mul(size));
}
#[no_mangle]
pub unsafe extern "C" fn ft_strlcpy(
	mut dst: *mut libc::c_char,
	mut src: *const libc::c_char,
	mut size: size_t,
) -> size_t {
	let mut srclen: size_t = 0;
	let mut x: size_t = 0;
	srclen = 0 as libc::c_int as size_t;
	while *src.offset(srclen as isize) as libc::c_int != '\0' as i32 {
		srclen = srclen.wrapping_add(1);
	}
	if size == 0 {
		return srclen;
	}
	x = -(1 as libc::c_int) as size_t;
	loop {
		x = x.wrapping_add(1);
		if !(*src.offset(x as isize) as libc::c_int != '\0' as i32
			&& x < size.wrapping_sub(1 as libc::c_int as libc::c_ulong))
		{
			break;
		}
		*dst.offset(x as isize) = *src.offset(x as isize);
	}
	if *src.offset(x as isize) == 0 || x == size.wrapping_sub(1 as libc::c_int as libc::c_ulong) {
		*dst.offset(x as isize) = 0 as libc::c_int as libc::c_char;
	}
	return srclen;
}
#[no_mangle]
pub unsafe extern "C" fn index_of(
	mut str: *mut libc::c_char,
	mut c: libc::c_char,
	mut max_len: libc::c_int,
) -> libc::c_int {
	let mut i: libc::c_int = 0;
	i = 0 as libc::c_int;
	while i < max_len
		&& *str.offset(i as isize) as libc::c_int != c as libc::c_int
		&& *str.offset(i as isize) as libc::c_int != '\0' as i32
	{
		i += 1;
	}
	return i;
}
