#![allow(unsafe_code)]
use crate::mem_compat::{memcpy, memset};
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::stdint_types::uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sha1_context_s {
    pub h0: uint32_t,
    pub h1: uint32_t,
    pub h2: uint32_t,
    pub h3: uint32_t,
    pub h4: uint32_t,
    pub nblocks: uint32_t,
    pub buf: [byte; 64],
    pub count: i32,
}
pub type sha1_context_t = sha1_context_s;
pub type sha1_digest_t = [byte; 20];
pub unsafe fn SHA1_Init(mut hd: *mut sha1_context_t) {
    (*hd).h0 = 0x67452301 as uint32_t;
    (*hd).h1 = 0xefcdab89_u32 as uint32_t;
    (*hd).h2 = 0x98badcfe_u32 as uint32_t;
    (*hd).h3 = 0x10325476 as uint32_t;
    (*hd).h4 = 0xc3d2e1f0_u32 as uint32_t;
    (*hd).nblocks = 0 as uint32_t;
    (*hd).count = 0_i32;
}
unsafe fn Transform(mut hd: *mut sha1_context_t, mut data: *mut byte) {
    let mut a: uint32_t = 0;
    let mut b: uint32_t = 0;
    let mut c: uint32_t = 0;
    let mut d: uint32_t = 0;
    let mut e: uint32_t = 0;
    let mut tm: uint32_t = 0;
    let mut x: [uint32_t; 16] = [0; 16];
    a = (*hd).h0;
    b = (*hd).h1;
    c = (*hd).h2;
    d = (*hd).h3;
    e = (*hd).h4;
    let mut i: i32 = 0;
    let mut p2: *mut byte = ::core::ptr::null_mut::<byte>();
    i = 0_i32;
    p2 = &raw mut x as *mut uint32_t as *mut byte;
    while i < 16_i32 {
        let fresh4 = data;
        data = data.offset(1);
        *p2.offset(3_i32 as isize) = *fresh4;
        let fresh5 = data;
        data = data.offset(1);
        *p2.offset(2_i32 as isize) = *fresh5;
        let fresh6 = data;
        data = data.offset(1);
        *p2.offset(1_i32 as isize) = *fresh6;
        let fresh7 = data;
        data = data.offset(1);
        *p2.offset(0_i32 as isize) = *fresh7;
        i += 1;
        p2 = p2.offset(4_i32 as isize);
    }
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(d ^ b & (c ^ d)) as i64
            + 0x5a827999_i64
            + x[0] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(c ^ a & (b ^ c)) as i64
            + 0x5a827999_i64
            + x[1] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(b ^ e & (a ^ b)) as i64
            + 0x5a827999_i64
            + x[2] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(a ^ d & (e ^ a)) as i64
            + 0x5a827999_i64
            + x[3] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(e ^ c & (d ^ e)) as i64
            + 0x5a827999_i64
            + x[4] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(d ^ b & (c ^ d)) as i64
            + 0x5a827999_i64
            + x[5] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(c ^ a & (b ^ c)) as i64
            + 0x5a827999_i64
            + x[6] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(b ^ e & (a ^ b)) as i64
            + 0x5a827999_i64
            + x[7] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(a ^ d & (e ^ a)) as i64
            + 0x5a827999_i64
            + x[8] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(e ^ c & (d ^ e)) as i64
            + 0x5a827999_i64
            + x[9] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(d ^ b & (c ^ d)) as i64
            + 0x5a827999_i64
            + x[10] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(c ^ a & (b ^ c)) as i64
            + 0x5a827999_i64
            + x[11] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(b ^ e & (a ^ b)) as i64
            + 0x5a827999_i64
            + x[12] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(a ^ d & (e ^ a)) as i64
            + 0x5a827999_i64
            + x[13] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(e ^ c & (d ^ e)) as i64
            + 0x5a827999_i64
            + x[14] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(d ^ b & (c ^ d)) as i64
            + 0x5a827999_i64
            + x[15] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(16_i32 & 0xf_i32) as usize]
        ^ x[((16_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((16_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((16_i32 - 3_i32) & 0xf_i32) as usize];
    x[(16_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(c ^ a & (b ^ c)) as i64
            + 0x5a827999_i64
            + x[(16_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(17_i32 & 0xf_i32) as usize]
        ^ x[((17_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((17_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((17_i32 - 3_i32) & 0xf_i32) as usize];
    x[(17_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(b ^ e & (a ^ b)) as i64
            + 0x5a827999_i64
            + x[(17_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(18_i32 & 0xf_i32) as usize]
        ^ x[((18_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((18_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((18_i32 - 3_i32) & 0xf_i32) as usize];
    x[(18_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(a ^ d & (e ^ a)) as i64
            + 0x5a827999_i64
            + x[(18_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(19_i32 & 0xf_i32) as usize]
        ^ x[((19_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((19_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((19_i32 - 3_i32) & 0xf_i32) as usize];
    x[(19_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(e ^ c & (d ^ e)) as i64
            + 0x5a827999_i64
            + x[(19_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(20_i32 & 0xf_i32) as usize]
        ^ x[((20_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((20_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((20_i32 - 3_i32) & 0xf_i32) as usize];
    x[(20_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0x6ed9eba1_i64
            + x[(20_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(21_i32 & 0xf_i32) as usize]
        ^ x[((21_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((21_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((21_i32 - 3_i32) & 0xf_i32) as usize];
    x[(21_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0x6ed9eba1_i64
            + x[(21_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(22_i32 & 0xf_i32) as usize]
        ^ x[((22_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((22_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((22_i32 - 3_i32) & 0xf_i32) as usize];
    x[(22_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0x6ed9eba1_i64
            + x[(22_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(23_i32 & 0xf_i32) as usize]
        ^ x[((23_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((23_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((23_i32 - 3_i32) & 0xf_i32) as usize];
    x[(23_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0x6ed9eba1_i64
            + x[(23_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(24_i32 & 0xf_i32) as usize]
        ^ x[((24_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((24_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((24_i32 - 3_i32) & 0xf_i32) as usize];
    x[(24_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0x6ed9eba1_i64
            + x[(24_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(25_i32 & 0xf_i32) as usize]
        ^ x[((25_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((25_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((25_i32 - 3_i32) & 0xf_i32) as usize];
    x[(25_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0x6ed9eba1_i64
            + x[(25_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(26_i32 & 0xf_i32) as usize]
        ^ x[((26_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((26_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((26_i32 - 3_i32) & 0xf_i32) as usize];
    x[(26_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0x6ed9eba1_i64
            + x[(26_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(27_i32 & 0xf_i32) as usize]
        ^ x[((27_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((27_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((27_i32 - 3_i32) & 0xf_i32) as usize];
    x[(27_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0x6ed9eba1_i64
            + x[(27_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(28_i32 & 0xf_i32) as usize]
        ^ x[((28_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((28_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((28_i32 - 3_i32) & 0xf_i32) as usize];
    x[(28_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0x6ed9eba1_i64
            + x[(28_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(29_i32 & 0xf_i32) as usize]
        ^ x[((29_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((29_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((29_i32 - 3_i32) & 0xf_i32) as usize];
    x[(29_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0x6ed9eba1_i64
            + x[(29_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(30_i32 & 0xf_i32) as usize]
        ^ x[((30_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((30_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((30_i32 - 3_i32) & 0xf_i32) as usize];
    x[(30_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0x6ed9eba1_i64
            + x[(30_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(31_i32 & 0xf_i32) as usize]
        ^ x[((31_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((31_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((31_i32 - 3_i32) & 0xf_i32) as usize];
    x[(31_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0x6ed9eba1_i64
            + x[(31_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(32_i32 & 0xf_i32) as usize]
        ^ x[((32_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((32_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((32_i32 - 3_i32) & 0xf_i32) as usize];
    x[(32_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0x6ed9eba1_i64
            + x[(32_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(33_i32 & 0xf_i32) as usize]
        ^ x[((33_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((33_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((33_i32 - 3_i32) & 0xf_i32) as usize];
    x[(33_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0x6ed9eba1_i64
            + x[(33_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(34_i32 & 0xf_i32) as usize]
        ^ x[((34_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((34_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((34_i32 - 3_i32) & 0xf_i32) as usize];
    x[(34_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0x6ed9eba1_i64
            + x[(34_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(35_i32 & 0xf_i32) as usize]
        ^ x[((35_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((35_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((35_i32 - 3_i32) & 0xf_i32) as usize];
    x[(35_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0x6ed9eba1_i64
            + x[(35_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(36_i32 & 0xf_i32) as usize]
        ^ x[((36_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((36_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((36_i32 - 3_i32) & 0xf_i32) as usize];
    x[(36_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0x6ed9eba1_i64
            + x[(36_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(37_i32 & 0xf_i32) as usize]
        ^ x[((37_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((37_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((37_i32 - 3_i32) & 0xf_i32) as usize];
    x[(37_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0x6ed9eba1_i64
            + x[(37_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(38_i32 & 0xf_i32) as usize]
        ^ x[((38_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((38_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((38_i32 - 3_i32) & 0xf_i32) as usize];
    x[(38_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0x6ed9eba1_i64
            + x[(38_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(39_i32 & 0xf_i32) as usize]
        ^ x[((39_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((39_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((39_i32 - 3_i32) & 0xf_i32) as usize];
    x[(39_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0x6ed9eba1_i64
            + x[(39_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(40_i32 & 0xf_i32) as usize]
        ^ x[((40_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((40_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((40_i32 - 3_i32) & 0xf_i32) as usize];
    x[(40_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b & c | d & (b | c)) as i64
            + 0x8f1bbcdc_i64
            + x[(40_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(41_i32 & 0xf_i32) as usize]
        ^ x[((41_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((41_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((41_i32 - 3_i32) & 0xf_i32) as usize];
    x[(41_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a & b | c & (a | b)) as i64
            + 0x8f1bbcdc_i64
            + x[(41_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(42_i32 & 0xf_i32) as usize]
        ^ x[((42_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((42_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((42_i32 - 3_i32) & 0xf_i32) as usize];
    x[(42_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e & a | b & (e | a)) as i64
            + 0x8f1bbcdc_i64
            + x[(42_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(43_i32 & 0xf_i32) as usize]
        ^ x[((43_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((43_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((43_i32 - 3_i32) & 0xf_i32) as usize];
    x[(43_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d & e | a & (d | e)) as i64
            + 0x8f1bbcdc_i64
            + x[(43_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(44_i32 & 0xf_i32) as usize]
        ^ x[((44_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((44_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((44_i32 - 3_i32) & 0xf_i32) as usize];
    x[(44_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c & d | e & (c | d)) as i64
            + 0x8f1bbcdc_i64
            + x[(44_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(45_i32 & 0xf_i32) as usize]
        ^ x[((45_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((45_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((45_i32 - 3_i32) & 0xf_i32) as usize];
    x[(45_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b & c | d & (b | c)) as i64
            + 0x8f1bbcdc_i64
            + x[(45_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(46_i32 & 0xf_i32) as usize]
        ^ x[((46_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((46_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((46_i32 - 3_i32) & 0xf_i32) as usize];
    x[(46_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a & b | c & (a | b)) as i64
            + 0x8f1bbcdc_i64
            + x[(46_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(47_i32 & 0xf_i32) as usize]
        ^ x[((47_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((47_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((47_i32 - 3_i32) & 0xf_i32) as usize];
    x[(47_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e & a | b & (e | a)) as i64
            + 0x8f1bbcdc_i64
            + x[(47_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(48_i32 & 0xf_i32) as usize]
        ^ x[((48_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((48_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((48_i32 - 3_i32) & 0xf_i32) as usize];
    x[(48_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d & e | a & (d | e)) as i64
            + 0x8f1bbcdc_i64
            + x[(48_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(49_i32 & 0xf_i32) as usize]
        ^ x[((49_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((49_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((49_i32 - 3_i32) & 0xf_i32) as usize];
    x[(49_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c & d | e & (c | d)) as i64
            + 0x8f1bbcdc_i64
            + x[(49_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(50_i32 & 0xf_i32) as usize]
        ^ x[((50_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((50_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((50_i32 - 3_i32) & 0xf_i32) as usize];
    x[(50_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b & c | d & (b | c)) as i64
            + 0x8f1bbcdc_i64
            + x[(50_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(51_i32 & 0xf_i32) as usize]
        ^ x[((51_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((51_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((51_i32 - 3_i32) & 0xf_i32) as usize];
    x[(51_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a & b | c & (a | b)) as i64
            + 0x8f1bbcdc_i64
            + x[(51_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(52_i32 & 0xf_i32) as usize]
        ^ x[((52_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((52_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((52_i32 - 3_i32) & 0xf_i32) as usize];
    x[(52_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e & a | b & (e | a)) as i64
            + 0x8f1bbcdc_i64
            + x[(52_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(53_i32 & 0xf_i32) as usize]
        ^ x[((53_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((53_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((53_i32 - 3_i32) & 0xf_i32) as usize];
    x[(53_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d & e | a & (d | e)) as i64
            + 0x8f1bbcdc_i64
            + x[(53_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(54_i32 & 0xf_i32) as usize]
        ^ x[((54_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((54_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((54_i32 - 3_i32) & 0xf_i32) as usize];
    x[(54_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c & d | e & (c | d)) as i64
            + 0x8f1bbcdc_i64
            + x[(54_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(55_i32 & 0xf_i32) as usize]
        ^ x[((55_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((55_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((55_i32 - 3_i32) & 0xf_i32) as usize];
    x[(55_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b & c | d & (b | c)) as i64
            + 0x8f1bbcdc_i64
            + x[(55_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(56_i32 & 0xf_i32) as usize]
        ^ x[((56_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((56_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((56_i32 - 3_i32) & 0xf_i32) as usize];
    x[(56_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a & b | c & (a | b)) as i64
            + 0x8f1bbcdc_i64
            + x[(56_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(57_i32 & 0xf_i32) as usize]
        ^ x[((57_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((57_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((57_i32 - 3_i32) & 0xf_i32) as usize];
    x[(57_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e & a | b & (e | a)) as i64
            + 0x8f1bbcdc_i64
            + x[(57_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(58_i32 & 0xf_i32) as usize]
        ^ x[((58_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((58_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((58_i32 - 3_i32) & 0xf_i32) as usize];
    x[(58_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d & e | a & (d | e)) as i64
            + 0x8f1bbcdc_i64
            + x[(58_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(59_i32 & 0xf_i32) as usize]
        ^ x[((59_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((59_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((59_i32 - 3_i32) & 0xf_i32) as usize];
    x[(59_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c & d | e & (c | d)) as i64
            + 0x8f1bbcdc_i64
            + x[(59_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(60_i32 & 0xf_i32) as usize]
        ^ x[((60_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((60_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((60_i32 - 3_i32) & 0xf_i32) as usize];
    x[(60_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0xca62c1d6_i64
            + x[(60_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(61_i32 & 0xf_i32) as usize]
        ^ x[((61_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((61_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((61_i32 - 3_i32) & 0xf_i32) as usize];
    x[(61_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0xca62c1d6_i64
            + x[(61_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(62_i32 & 0xf_i32) as usize]
        ^ x[((62_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((62_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((62_i32 - 3_i32) & 0xf_i32) as usize];
    x[(62_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0xca62c1d6_i64
            + x[(62_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(63_i32 & 0xf_i32) as usize]
        ^ x[((63_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((63_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((63_i32 - 3_i32) & 0xf_i32) as usize];
    x[(63_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0xca62c1d6_i64
            + x[(63_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(64_i32 & 0xf_i32) as usize]
        ^ x[((64_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((64_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((64_i32 - 3_i32) & 0xf_i32) as usize];
    x[(64_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0xca62c1d6_i64
            + x[(64_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(65_i32 & 0xf_i32) as usize]
        ^ x[((65_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((65_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((65_i32 - 3_i32) & 0xf_i32) as usize];
    x[(65_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0xca62c1d6_i64
            + x[(65_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(66_i32 & 0xf_i32) as usize]
        ^ x[((66_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((66_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((66_i32 - 3_i32) & 0xf_i32) as usize];
    x[(66_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0xca62c1d6_i64
            + x[(66_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(67_i32 & 0xf_i32) as usize]
        ^ x[((67_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((67_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((67_i32 - 3_i32) & 0xf_i32) as usize];
    x[(67_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0xca62c1d6_i64
            + x[(67_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(68_i32 & 0xf_i32) as usize]
        ^ x[((68_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((68_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((68_i32 - 3_i32) & 0xf_i32) as usize];
    x[(68_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0xca62c1d6_i64
            + x[(68_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(69_i32 & 0xf_i32) as usize]
        ^ x[((69_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((69_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((69_i32 - 3_i32) & 0xf_i32) as usize];
    x[(69_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0xca62c1d6_i64
            + x[(69_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(70_i32 & 0xf_i32) as usize]
        ^ x[((70_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((70_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((70_i32 - 3_i32) & 0xf_i32) as usize];
    x[(70_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0xca62c1d6_i64
            + x[(70_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(71_i32 & 0xf_i32) as usize]
        ^ x[((71_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((71_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((71_i32 - 3_i32) & 0xf_i32) as usize];
    x[(71_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0xca62c1d6_i64
            + x[(71_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(72_i32 & 0xf_i32) as usize]
        ^ x[((72_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((72_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((72_i32 - 3_i32) & 0xf_i32) as usize];
    x[(72_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0xca62c1d6_i64
            + x[(72_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(73_i32 & 0xf_i32) as usize]
        ^ x[((73_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((73_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((73_i32 - 3_i32) & 0xf_i32) as usize];
    x[(73_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0xca62c1d6_i64
            + x[(73_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(74_i32 & 0xf_i32) as usize]
        ^ x[((74_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((74_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((74_i32 - 3_i32) & 0xf_i32) as usize];
    x[(74_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0xca62c1d6_i64
            + x[(74_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    tm = x[(75_i32 & 0xf_i32) as usize]
        ^ x[((75_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((75_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((75_i32 - 3_i32) & 0xf_i32) as usize];
    x[(75_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    e = (e as i64
        + ((a << 5_i32 | a >> (32_i32 - 5_i32)).wrapping_add(b ^ c ^ d) as i64
            + 0xca62c1d6_i64
            + x[(75_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    b = b << 30_i32 | b >> (32_i32 - 30_i32);
    tm = x[(76_i32 & 0xf_i32) as usize]
        ^ x[((76_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((76_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((76_i32 - 3_i32) & 0xf_i32) as usize];
    x[(76_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    d = (d as i64
        + ((e << 5_i32 | e >> (32_i32 - 5_i32)).wrapping_add(a ^ b ^ c) as i64
            + 0xca62c1d6_i64
            + x[(76_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    a = a << 30_i32 | a >> (32_i32 - 30_i32);
    tm = x[(77_i32 & 0xf_i32) as usize]
        ^ x[((77_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((77_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((77_i32 - 3_i32) & 0xf_i32) as usize];
    x[(77_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    c = (c as i64
        + ((d << 5_i32 | d >> (32_i32 - 5_i32)).wrapping_add(e ^ a ^ b) as i64
            + 0xca62c1d6_i64
            + x[(77_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    e = e << 30_i32 | e >> (32_i32 - 30_i32);
    tm = x[(78_i32 & 0xf_i32) as usize]
        ^ x[((78_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((78_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((78_i32 - 3_i32) & 0xf_i32) as usize];
    x[(78_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    b = (b as i64
        + ((c << 5_i32 | c >> (32_i32 - 5_i32)).wrapping_add(d ^ e ^ a) as i64
            + 0xca62c1d6_i64
            + x[(78_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    d = d << 30_i32 | d >> (32_i32 - 30_i32);
    tm = x[(79_i32 & 0xf_i32) as usize]
        ^ x[((79_i32 - 14_i32) & 0xf_i32) as usize]
        ^ x[((79_i32 - 8_i32) & 0xf_i32) as usize]
        ^ x[((79_i32 - 3_i32) & 0xf_i32) as usize];
    x[(79_i32 & 0xf_i32) as usize] = tm << 1_i32 | tm >> (32_i32 - 1_i32);
    a = (a as i64
        + ((b << 5_i32 | b >> (32_i32 - 5_i32)).wrapping_add(c ^ d ^ e) as i64
            + 0xca62c1d6_i64
            + x[(79_i32 & 0xf_i32) as usize] as i64)) as uint32_t;
    c = c << 30_i32 | c >> (32_i32 - 30_i32);
    (*hd).h0 = (*hd).h0.wrapping_add(a);
    (*hd).h1 = (*hd).h1.wrapping_add(b);
    (*hd).h2 = (*hd).h2.wrapping_add(c);
    (*hd).h3 = (*hd).h3.wrapping_add(d);
    (*hd).h4 = (*hd).h4.wrapping_add(e);
}
pub unsafe fn SHA1_Update(mut hd: *mut sha1_context_t, mut inbuf: *mut byte, mut inlen: size_t) {
    if (*hd).count == 64_i32 {
        Transform(hd, &raw mut (*hd).buf as *mut byte);
        (*hd).count = 0_i32;
        (*hd).nblocks = (*hd).nblocks.wrapping_add(1);
    }
    if inbuf.is_null() {
        return;
    }
    if (*hd).count != 0 {
        while inlen != 0 && (*hd).count < 64_i32 {
            let fresh0 = inbuf;
            inbuf = inbuf.offset(1);
            let fresh1 = (*hd).count;
            (*hd).count += 1;
            (*hd).buf[fresh1 as usize] = *fresh0;
            inlen = inlen.wrapping_sub(1);
        }
        SHA1_Update(hd, ::core::ptr::null_mut::<byte>(), 0 as size_t);
        if inlen == 0 {
            return;
        }
    }
    while inlen >= 64 as size_t {
        Transform(hd, inbuf);
        (*hd).count = 0_i32;
        (*hd).nblocks = (*hd).nblocks.wrapping_add(1);
        inlen = inlen.wrapping_sub(64 as size_t);
        inbuf = inbuf.offset(64_i32 as isize);
    }
    while inlen != 0 && (*hd).count < 64_i32 {
        let fresh2 = inbuf;
        inbuf = inbuf.offset(1);
        let fresh3 = (*hd).count;
        (*hd).count += 1;
        (*hd).buf[fresh3 as usize] = *fresh2;
        inlen = inlen.wrapping_sub(1);
    }
}
pub unsafe fn SHA1_Final(mut digest: *mut byte, mut hd: *mut sha1_context_t) {
    let mut t: uint32_t = 0;
    let mut msb: uint32_t = 0;
    let mut lsb: uint32_t = 0;
    let mut p: *mut byte = ::core::ptr::null_mut::<byte>();
    SHA1_Update(hd, ::core::ptr::null_mut::<byte>(), 0 as size_t);
    t = (*hd).nblocks;
    lsb = t << 6_i32;
    msb = t >> 26_i32;
    t = lsb;
    lsb = lsb.wrapping_add((*hd).count as uint32_t);
    if lsb < t {
        msb = msb.wrapping_add(1);
    }
    t = lsb;
    lsb <<= 3_i32;
    msb <<= 3_i32;
    msb |= t >> 29_i32;
    if (*hd).count < 56_i32 {
        let fresh8 = (*hd).count;
        (*hd).count += 1;
        (*hd).buf[fresh8 as usize] = 0x80 as byte;
        while (*hd).count < 56_i32 {
            let fresh9 = (*hd).count;
            (*hd).count += 1;
            (*hd).buf[fresh9 as usize] = 0 as byte;
        }
    } else {
        let fresh10 = (*hd).count;
        (*hd).count += 1;
        (*hd).buf[fresh10 as usize] = 0x80 as byte;
        while (*hd).count < 64_i32 {
            let fresh11 = (*hd).count;
            (*hd).count += 1;
            (*hd).buf[fresh11 as usize] = 0 as byte;
        }
        SHA1_Update(hd, ::core::ptr::null_mut::<byte>(), 0 as size_t);
        memset(
            &raw mut (*hd).buf as *mut byte as *mut ::core::ffi::c_void,
            0_i32,
            56 as size_t,
        );
    }
    (*hd).buf[56] = (msb >> 24_i32) as byte;
    (*hd).buf[57] = (msb >> 16_i32) as byte;
    (*hd).buf[58] = (msb >> 8_i32) as byte;
    (*hd).buf[59] = msb as byte;
    (*hd).buf[60] = (lsb >> 24_i32) as byte;
    (*hd).buf[61] = (lsb >> 16_i32) as byte;
    (*hd).buf[62] = (lsb >> 8_i32) as byte;
    (*hd).buf[63] = lsb as byte;
    Transform(hd, &raw mut (*hd).buf as *mut byte);
    p = &raw mut (*hd).buf as *mut byte;
    let fresh12 = p;
    p = p.offset(1);
    *fresh12 = ((*hd).h0 >> 24_i32) as byte;
    let fresh13 = p;
    p = p.offset(1);
    *fresh13 = ((*hd).h0 >> 16_i32) as byte;
    let fresh14 = p;
    p = p.offset(1);
    *fresh14 = ((*hd).h0 >> 8_i32) as byte;
    let fresh15 = p;
    p = p.offset(1);
    *fresh15 = (*hd).h0 as byte;
    let fresh16 = p;
    p = p.offset(1);
    *fresh16 = ((*hd).h1 >> 24_i32) as byte;
    let fresh17 = p;
    p = p.offset(1);
    *fresh17 = ((*hd).h1 >> 16_i32) as byte;
    let fresh18 = p;
    p = p.offset(1);
    *fresh18 = ((*hd).h1 >> 8_i32) as byte;
    let fresh19 = p;
    p = p.offset(1);
    *fresh19 = (*hd).h1 as byte;
    let fresh20 = p;
    p = p.offset(1);
    *fresh20 = ((*hd).h2 >> 24_i32) as byte;
    let fresh21 = p;
    p = p.offset(1);
    *fresh21 = ((*hd).h2 >> 16_i32) as byte;
    let fresh22 = p;
    p = p.offset(1);
    *fresh22 = ((*hd).h2 >> 8_i32) as byte;
    let fresh23 = p;
    p = p.offset(1);
    *fresh23 = (*hd).h2 as byte;
    let fresh24 = p;
    p = p.offset(1);
    *fresh24 = ((*hd).h3 >> 24_i32) as byte;
    let fresh25 = p;
    p = p.offset(1);
    *fresh25 = ((*hd).h3 >> 16_i32) as byte;
    let fresh26 = p;
    p = p.offset(1);
    *fresh26 = ((*hd).h3 >> 8_i32) as byte;
    let fresh27 = p;
    p = p.offset(1);
    *fresh27 = (*hd).h3 as byte;
    let fresh28 = p;
    p = p.offset(1);
    *fresh28 = ((*hd).h4 >> 24_i32) as byte;
    let fresh29 = p;
    p = p.offset(1);
    *fresh29 = ((*hd).h4 >> 16_i32) as byte;
    let fresh30 = p;
    p = p.offset(1);
    *fresh30 = ((*hd).h4 >> 8_i32) as byte;
    let fresh31 = p;
    p = p.offset(1);
    *fresh31 = (*hd).h4 as byte;
    memcpy(
        digest as *mut ::core::ffi::c_void,
        &raw mut (*hd).buf as *mut byte as *const ::core::ffi::c_void,
        ::core::mem::size_of::<sha1_digest_t>() as size_t,
    );
}
pub unsafe fn SHA1_UpdateInt32(mut context: *mut sha1_context_t, mut val: u32) {
    let mut buf: [byte; 4] = [0; 4];
    buf[0] = (val >> 24_i32 & 0xff_u32) as byte;
    buf[1] = (val >> 16_i32 & 0xff_u32) as byte;
    buf[2] = (val >> 8_i32 & 0xff_u32) as byte;
    buf[3] = (val & 0xff_u32) as byte;
    SHA1_Update(context, &raw mut buf as *mut byte, 4 as size_t);
}
pub unsafe fn SHA1_UpdateString(
    mut context: *mut sha1_context_t,
    mut str: *mut ::core::ffi::c_char,
) {
    SHA1_Update(
        context,
        str as *mut byte,
        ::std::ffi::CStr::from_ptr(str as *const ::core::ffi::c_char)
            .to_bytes()
            .len()
            .wrapping_add(1 as size_t),
    );
}

/// Safe streaming interface over the C-style routines above (this file is the
/// engine's deliberate unsafe exception; callers elsewhere stay unsafe-free).
pub struct Sha1 {
    ctx: sha1_context_t,
}
impl Sha1 {
    pub fn new() -> Self {
        let mut ctx = sha1_context_s {
            h0: 0,
            h1: 0,
            h2: 0,
            h3: 0,
            h4: 0,
            nblocks: 0,
            buf: [0; 64],
            count: 0,
        };
        unsafe { SHA1_Init(&mut ctx) };
        Sha1 { ctx }
    }

    pub fn update_int32(&mut self, val: u32) {
        unsafe { SHA1_UpdateInt32(&mut self.ctx, val) };
    }

    /// Hashes `s` followed by its NUL terminator, like the C `SHA1_UpdateString`.
    pub fn update_string(&mut self, s: &[u8]) {
        let mut bytes = s.to_vec();
        bytes.push(0);
        unsafe { SHA1_Update(&mut self.ctx, bytes.as_mut_ptr(), bytes.len() as size_t) };
    }

    pub fn finalize(mut self) -> sha1_digest_t {
        let mut digest: sha1_digest_t = [0; 20];
        unsafe { SHA1_Final(digest.as_mut_ptr(), &mut self.ctx) };
        digest
    }
}
