/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[rustfmt::skip]
pub(crate) unsafe fn dup_mm_setr_epi8(e: [i8; 16]) -> __m256i {
    _mm256_setr_epi8(
        e[0x0], e[0x1], e[0x2], e[0x3], e[0x4], e[0x5], e[0x6], e[0x7], 
        e[0x8], e[0x9], e[0xa], e[0xb], e[0xc], e[0xd], e[0xe], e[0xf], 
        e[0x0], e[0x1], e[0x2], e[0x3], e[0x4], e[0x5], e[0x6], e[0x7], 
        e[0x8], e[0x9], e[0xa], e[0xb], e[0xc], e[0xd], e[0xe], e[0xf], 
    )
}

#[rustfmt::skip]
pub(crate) unsafe fn dup_mm_setr_epu8(e: [u8; 16]) -> __m256i {
    _mm256_setr_epi8(
        e[0x0] as _, e[0x1] as _, e[0x2] as _, e[0x3] as _, e[0x4] as _, e[0x5] as _, e[0x6] as _, e[0x7] as _, 
        e[0x8] as _, e[0x9] as _, e[0xa] as _, e[0xb] as _, e[0xc] as _, e[0xd] as _, e[0xe] as _, e[0xf] as _, 
        e[0x0] as _, e[0x1] as _, e[0x2] as _, e[0x3] as _, e[0x4] as _, e[0x5] as _, e[0x6] as _, e[0x7] as _, 
        e[0x8] as _, e[0x9] as _, e[0xa] as _, e[0xb] as _, e[0xc] as _, e[0xd] as _, e[0xe] as _, e[0xf] as _, 
    )
}

pub(crate) unsafe fn _mm256_not_si256(i: __m256i) -> __m256i {
    _mm256_xor_si256(i, _mm256_set1_epi8(!0))
}

pub(crate) unsafe fn array_as_m256i(v: [u8; 32]) -> __m256i {
    core::mem::transmute(v)
}

pub(crate) unsafe fn m256i_as_array(v: __m256i) -> [u8; 32] {
    core::mem::transmute(v)
}
