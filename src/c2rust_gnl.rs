#![allow(
	dead_code,
	mutable_transmutes,
	non_camel_case_types,
	non_snake_case,
	non_upper_case_globals,
	unused_assignments,
	unused_mut,
	static_mut_refs
)]

pub const BUF_SIZE_MINUS: usize = 1000000;
const BUF_SIZE: usize = BUF_SIZE_MINUS + 1;
unsafe extern "C" {
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
	fn free(_: *mut libc::c_void);
	fn read(__fd: libc::c_int, __buf: *mut libc::c_void, __nbytes: size_t) -> ssize_t;
}
pub type size_t = libc::c_ulong;
pub type __ssize_t = libc::c_long;
pub type ssize_t = __ssize_t;
#[unsafe(no_mangle)]
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
		i;
	}
	return i;
}
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
		srclen;
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
unsafe extern "C" fn check_free(
	mut buf: *mut libc::c_char,
	mut buf_idx: libc::c_int,
	mut line: *mut libc::c_char,
	mut is_buf: bool,
) -> *mut libc::c_char {
	let mut buf_nl_idx: libc::c_int = 0;
	let mut ret: *mut libc::c_char = 0 as *mut libc::c_char;
	let mut gnl_idx: libc::c_int = 0;
	if line.is_null() {
		return 0 as *mut libc::c_char;
	}
	if is_buf {
		buf_nl_idx = index_of(buf, '\n' as i32 as libc::c_char, 2147483647 as libc::c_int);
		ft_memcpy(
			line as *mut libc::c_void,
			buf as *const libc::c_void,
			(buf_idx + 1 as libc::c_int) as size_t,
		);
		if *buf.offset(buf_nl_idx as isize) as libc::c_int != '\n' as i32 {
			*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
		} else {
			buf_nl_idx += 1;
			buf_nl_idx;
		}
		ft_memcpy(
			buf as *mut libc::c_void,
			buf.offset(buf_nl_idx as isize) as *const libc::c_void,
			(BUF_SIZE_MINUS as libc::c_int - buf_nl_idx + 1 as libc::c_int) as size_t,
		);
	}
	gnl_idx = index_of(line, '\n' as i32 as libc::c_char, 2147483647 as libc::c_int);
	if *line.offset(gnl_idx as isize) as libc::c_int == '\n' as i32 {
		gnl_idx += 1;
		gnl_idx;
	}
	ret = ft_calloc(
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		(gnl_idx + 1 as libc::c_int) as size_t,
	) as *mut libc::c_char;
	if ret.is_null() {
		free(line as *mut libc::c_void);
		return 0 as *mut libc::c_void as *mut libc::c_char;
	}
	ft_memcpy(
		ret as *mut libc::c_void,
		line as *const libc::c_void,
		gnl_idx as size_t,
	);
	free(line as *mut libc::c_void);
	return ret;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn c2rust_gnl(mut fd: libc::c_int) -> *mut libc::c_char {
	let mut line: *mut libc::c_char = 0 as *mut libc::c_char;
	static mut buf: [libc::c_char; BUF_SIZE] = [0; BUF_SIZE];
	let mut buf_idx: libc::c_int = 0;
	if fd < 0 as libc::c_int || (BUF_SIZE_MINUS as libc::c_int) < 1 as libc::c_int {
		return 0 as *mut libc::c_char;
	}
	line = 0 as *mut libc::c_char;
	buf_idx = -(1 as libc::c_int);
	loop {
		buf_idx += 1;
		if !(buf_idx < BUF_SIZE_MINUS as libc::c_int && buf[buf_idx as usize] as libc::c_int != 0) {
			break;
		}
		if buf[buf_idx as usize] as libc::c_int == '\n' as i32 {
			line = ft_calloc(
				::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
				(BUF_SIZE_MINUS as libc::c_int + 1 as libc::c_int) as size_t,
			) as *mut libc::c_char;
			if line.is_null() {
				return 0 as *mut libc::c_char;
			}
			return check_free(buf.as_mut_ptr(), buf_idx, line, 1 as libc::c_int != 0);
		}
	}
	if buf[buf_idx as usize] as libc::c_int != '\n' as i32 {
		read_line(buf.as_mut_ptr(), fd, &mut buf_idx, &mut line);
	}
	return check_free(buf.as_mut_ptr(), buf_idx, line, 0 as libc::c_int != 0);
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
		BUF_SIZE_MINUS as libc::c_int as size_t,
	);
	buf_nl_idx = index_of(
		buf,
		'\n' as i32 as libc::c_char,
		BUF_SIZE_MINUS as libc::c_int + 1 as libc::c_int,
	);
	if *buf.offset(buf_nl_idx as isize) as libc::c_int != '\n' as i32 {
		*buf.offset(buf_nl_idx as isize) = 0 as libc::c_int as libc::c_char;
	} else {
		buf_nl_idx += 1;
		buf_nl_idx;
	}
	ft_memcpy(
		buf as *mut libc::c_void,
		buf.offset(buf_nl_idx as isize) as *const libc::c_void,
		(BUF_SIZE_MINUS as libc::c_int - buf_nl_idx + 1 as libc::c_int) as size_t,
	);
	return 1 as libc::c_int != 0;
}
unsafe extern "C" fn read_line(
	mut buf: *mut libc::c_char,
	mut fd: libc::c_int,
	mut buf_idx: *mut libc::c_int,
	mut line: *mut *mut libc::c_char,
) -> *mut libc::c_char {
	let mut tmp: [libc::c_char; BUF_SIZE] = [0; BUF_SIZE];
	let rd: libc::c_int = read(
		fd,
		ft_memset(
			tmp.as_mut_ptr() as *mut libc::c_void,
			0 as libc::c_int,
			BUF_SIZE_MINUS as libc::c_int as size_t,
		),
		BUF_SIZE_MINUS as libc::c_int as size_t,
	) as libc::c_int;
	let mut tmp_nl_idx: libc::c_int = 0;
	if rd == -(1 as libc::c_int) {
		return ft_memset(
			buf as *mut libc::c_void,
			0 as libc::c_int,
			BUF_SIZE_MINUS as libc::c_int as size_t,
		) as *mut libc::c_char;
	}
	if rd > 0 as libc::c_int {
		*buf_idx += BUF_SIZE_MINUS as libc::c_int;
	}
	tmp_nl_idx = index_of(
		tmp.as_mut_ptr(),
		'\n' as i32 as libc::c_char,
		BUF_SIZE_MINUS as libc::c_int,
	);
	if (tmp[tmp_nl_idx as usize] as libc::c_int == '\n' as i32
		|| rd == 0 as libc::c_int && *buf_idx != 0 as libc::c_int)
		&& !iter_line(line, buf, tmp.as_mut_ptr(), *buf_idx)
	{
		return 0 as *mut libc::c_char;
	}
	if tmp[tmp_nl_idx as usize] as libc::c_int != '\n' as i32
		&& rd != 0 as libc::c_int
		&& (read_line(buf, fd, buf_idx, line)).is_null()
	{
		return 0 as *mut libc::c_char;
	}
	if rd > 0 as libc::c_int && *buf_idx != 0 as libc::c_int {
		*buf_idx -= BUF_SIZE_MINUS as libc::c_int;
		tmp_nl_idx = index_of(
			tmp.as_mut_ptr(),
			'\n' as i32 as libc::c_char,
			BUF_SIZE_MINUS as libc::c_int,
		);
		ft_memcpy(
			(*line).offset(*buf_idx as isize) as *mut libc::c_void,
			tmp.as_mut_ptr() as *const libc::c_void,
			tmp_nl_idx as size_t,
		);
		if tmp[tmp_nl_idx as usize] as libc::c_int == '\n' as i32 {
			*(*line).offset((*buf_idx + tmp_nl_idx) as isize) = '\n' as i32 as libc::c_char;
		}
	}
	return *line;
}
