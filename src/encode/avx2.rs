/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::avx2::*;

/// # Safety
/// The caller should ensure the requisite CPU features are enabled.
#[target_feature(enable = "avx2")]
unsafe fn encode_block(block: &mut <Avx2 as super::Encoder>::Block, charset: crate::CharacterSet) {
    let input = array_as_m256i(*block);

    // The general idea is to recognize that the 6-bit input can fall in one of
    // five output ranges: uppercase letters, lowercase letters, numbers,
    // special character 1, special character 2. First we calculate what range
    // the input is in, then we determine what value would need to be added to
    // arrive at the right ASCII output value, and add it.

    // Check whether the input should be encoded as a letter.
    //
    // If it should, result is now 0. Otherwise, it's in the range 1...12,
    // inclusive.
    let result = _mm256_subs_epu8(input, _mm256_set1_epi8(51));

    // Check whether the input should be encoded as an uppercase letter.
    //
    // If it should, result is now 0xff. Otherwise, it's 0.
    let less = _mm256_cmpgt_epi8(_mm256_set1_epi8(26), input);

    // Choose one of the 5 ranges for each input.
    //
    // 0: lowercase letter
    // 1...10: number
    // 11: special character 1
    // 12: special character 2
    // 13: uppercase letter
    let result = _mm256_or_si256(result, _mm256_and_si256(less, _mm256_set1_epi8(13)));

    // Choose the lookup table based on the character set.
    //
    // The lookup table gives the amount that needs to be added to the input
    // (AKA the shift) to convert it to the appropriate ASCII code.
    let shift_lut = match charset {
        crate::CharacterSet::Standard => dup_mm_setr_epi8([
            b'a' as i8 - 26,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'+' as i8 - 62,
            b'/' as i8 - 63,
            b'A' as _,
            0,
            0,
        ]),
        crate::CharacterSet::UrlSafe => dup_mm_setr_epi8([
            b'a' as i8 - 26,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'0' as i8 - 52,
            b'-' as i8 - 62,
            b'_' as i8 - 63,
            b'A' as _,
            0,
            0,
        ]),
    };

    let shift = _mm256_shuffle_epi8(shift_lut, result);

    *block = m256i_as_array(_mm256_add_epi8(shift, input));
}

/// # Safety
/// The caller should ensure the requisite CPU features are enabled.
#[target_feature(enable = "avx2")]
unsafe fn unpack_block(
    input: &<Avx2 as super::Unpacker>::Input,
    output: &mut <Avx2 as super::Unpacker>::Output,
) {
    let input = _mm256_set_m128i(
        core::ptr::read_unaligned(input.as_ptr().offset(8) as _),
        core::ptr::read_unaligned(input.as_ptr() as _),
    );

    #[rustfmt::skip]
    let shuf = _mm256_set_epi8(
        14, 15, 13, 14,
        11, 12, 10, 11,
         8,  9,  7,  8,
         5,  6,  4,  5,

        10, 11, 9, 10,
         7,  8, 6,  7,
         4,  5, 3,  4,
         1,  2, 0,  1,
    );

    let input = _mm256_shuffle_epi8(input, shuf);

    let t0 = _mm256_and_si256(input, _mm256_set1_epi32(0x0fc0fc00));
    let t1 = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));
    let t2 = _mm256_and_si256(input, _mm256_set1_epi32(0x003f03f0));
    let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));
    *output = m256i_as_array(_mm256_or_si256(t1, t3));
}

#[derive(Copy, Clone)]
pub(super) struct Avx2 {
    _private: (),
}

impl Avx2 {
    /// # Safety
    /// The caller should ensure the requisite CPU features are enabled.
    #[target_feature(enable = "avx2,bmi1,sse4.2,popcnt")]
    pub(super) unsafe fn new() -> Avx2 {
        Avx2 { _private: () }
    }
}

impl super::Encoder for Avx2 {
    type Block = [u8; 32];

    fn encode_block(self, block: &mut Self::Block, charset: crate::CharacterSet) {
        // safe: `self` was given as a witness that the features are available
        unsafe { encode_block(block, charset) }
    }
}

impl super::Unpacker for Avx2 {
    type Input = [u8; 24];
    type Output = [u8; 32];

    fn unpack_block(self, input: &Self::Input, output: &mut Self::Output) {
        // safe: `self` was given as a witness that the features are available
        unsafe { unpack_block(input, output) }
    }
}
