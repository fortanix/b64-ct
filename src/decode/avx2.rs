/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::arch::x86::*;

use crate::avx2::*;

/// # Safety
/// The caller should ensure the requisite CPU features are enabled.
#[target_feature(enable = "avx2,bmi1,sse4.2,popcnt")]
unsafe fn decode_avx2(input: __m256i) -> (__m256i, u32, u32) {
    // Step 0. Split input bytes into nibbles.
    let higher_nibble = _mm256_and_si256(_mm256_srli_epi16(input, 4), _mm256_set1_epi8(0x0f));
    let lower_nibble = _mm256_and_si256(input, _mm256_set1_epi8(0x0f));

    // Step 1. Find invalid characters. Steps 2 & 3 will compute invalid 6-bit
    // values for invalid characters. The result of the computation should only
    // be used if no invalid characters are found.

    // This table contains 128 bits, one bit for each of the lower 128 ASCII
    // characters. A set bit indicates that the character is in the base64
    // character set (the character is valid) or the character is considered
    // ASCII whitespace. This table is indexed by ASCII low nibble.
    #[rustfmt::skip]
    let row_lut = dup_mm_setr_epu8([
        0b1010_1100, 0b1111_1000, 0b1111_1000, 0b1111_1000, 
        0b1111_1000, 0b1111_1000, 0b1111_1000, 0b1111_1000, 
        0b1111_1000, 0b1111_1001, 0b1111_0001, 0b0101_0100, 
        0b0101_0001, 0b0101_0101, 0b0101_0000, 0b0111_0100,
    ]);

    // This table contains column offsets (within a byte) for the table above.
    // This table is indexed by ASCII high nibble.
    #[rustfmt::skip]
    let column_lut = dup_mm_setr_epu8([
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
           0,    0,    0,    0,    0,    0,    0,    0,
    ]);

    // Lookup table row
    let row = _mm256_shuffle_epi8(row_lut, lower_nibble);
    // Lookup column offset
    let column = _mm256_shuffle_epi8(column_lut, higher_nibble);
    // Lookup valid characters
    let valid = _mm256_and_si256(row, column);
    // Compute invalid character mask
    let non_match = _mm256_cmpeq_epi8(valid, _mm256_setzero_si256());
    // Transform mask to u32
    let invalid_mask = _mm256_movemask_epi8(non_match);

    // Step 2. Numbers & letters: compute 6-bit value for the 3 different
    // ranges by simply adjusting the ASCII value.

    // This table contains the offsets for the alphanumerical ASCII ranges.
    // This table is indexed by ASCII high nibble.
    #[rustfmt::skip]
    let shift_lut = dup_mm_setr_epi8([
        0, 0, 0,
        // '0' through '9'
        4,
        // 'A' through 'Z'
        -65, -65,
        // 'a' through 'z'
        -71, -71,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    // Get offset
    let shift = _mm256_shuffle_epi8(shift_lut, higher_nibble);
    // Compute 6-bit value
    let shifted = _mm256_add_epi8(input, shift);

    // Step 3. Special characters: lookup 6-bit value by looking it up in a
    // table.

    // This table specifies the ASCII ranges that contain valid special
    // characters. This table is indexed by ASCII high nibble.
    #[rustfmt::skip]
    let spcrange_lut = dup_mm_setr_epu8([
        0, 0, 0xff, 0, 0, 0xff, 0, 0,
        0, 0,    0, 0, 0,    0, 0, 0,
    ]);

    // This table specifies the (inverted) 6-bit values for the special
    // characters. The values in this table act as both a value and a blend
    // mask. This table is indexed by the difference between ASCII low and high
    // nibble.
    #[rustfmt::skip]
    let spcchar_lut = dup_mm_setr_epu8([
        0,   0,   0,   0, 0,   0, 0, 0,
        // '+', '_', '-',    '/'
        0, !62, !63, !62, 0, !63, 0, 0,
    ]);

    // Check if character is in the range for special characters
    let sel_range = _mm256_shuffle_epi8(spcrange_lut, higher_nibble);
    // Compute difference between ASCII low and high nibble
    let lo_sub_hi = _mm256_sub_epi8(lower_nibble, higher_nibble);
    // Lookup special character 6-bit value
    let specials = _mm256_shuffle_epi8(spcchar_lut, lo_sub_hi);
    // Combine blend masks from range and value
    let sel_spec = _mm256_and_si256(sel_range, specials);

    // Combine results of step 1 and step 2
    let result = _mm256_blendv_epi8(shifted, _mm256_not_si256(specials), sel_spec);

    // Step 4. Compute mask for valid non-whitespace bytes. The mask will be
    // used to copy only relevant bytes into the output.

    // This table specifies the character ranges which should be decoded. The
    // format is a range table for the PCMPESTRM instruction.
    #[rustfmt::skip]
    let valid_nonws_set = _mm_setr_epi8(
        b'A' as _, b'Z' as _,
        b'a' as _, b'z' as _,
        b'0' as _, b'9' as _,
        b'+' as _, b'+' as _,
        b'/' as _, b'/' as _,
        b'-' as _, b'-' as _,
        b'_' as _, b'_' as _,
        0, 0,
    );

    // Split input into 128-bit values
    let lane0 = _mm256_extracti128_si256(input, 0);
    let lane1 = _mm256_extracti128_si256(input, 1);
    // Compute bitmask for each 128-bit value
    const CMP_FLAGS: i32 = _SIDD_UBYTE_OPS | _SIDD_CMP_RANGES | _SIDD_BIT_MASK;
    let mask0 = _mm_cmpestrm(valid_nonws_set, 14, lane0, 16, CMP_FLAGS);
    let mask1 = _mm_cmpestrm(valid_nonws_set, 14, lane1, 16, CMP_FLAGS);

    // Combine bitmasks into integer value
    let first = _mm_extract_epi16(mask0, 0) as u16;
    let second = _mm_extract_epi16(mask1, 0) as u16;
    let valid_mask = first as u32 + ((second as u32) << 16);

    (result, invalid_mask as _, valid_mask as _)
}

/// # Safety
/// The caller should ensure the requisite CPU features are enabled.
#[target_feature(enable = "avx2,bmi1,sse4.2,popcnt")]
unsafe fn decode_block(block: &mut <Avx2 as super::Decoder>::Block) -> super::BlockResult {
    let input = array_as_m256i(*block);

    let (unpacked, invalid_mask, mut valid_mask) = decode_avx2(input);

    let unpacked = m256i_as_array(unpacked);

    let first_invalid = match invalid_mask.trailing_zeros() {
        32 => None,
        v => Some(v as _),
    };
    let out_length = valid_mask.count_ones() as _;

    let mut out_iter = block.iter_mut();
    // TODO: Optimize loop (https://github.com/fortanix/b64-ct/issues/2)
    for &val in unpacked.iter() {
        if (valid_mask & 1) == 1 {
            *out_iter.next().unwrap() = val;
        }
        valid_mask >>= 1;
    }

    super::BlockResult {
        out_length,
        first_invalid,
    }
}

/// # Safety
/// The caller should ensure the requisite CPU features are enabled.
#[target_feature(enable = "avx2,bmi1,sse4.2,popcnt")]
unsafe fn pack_block(input: &<Avx2 as super::Packer>::Input, output: &mut [u8]) {
    assert_eq!(output.len(), <Avx2 as super::Packer>::OUT_BUF_LEN);

    let unpacked = array_as_m256i(*input);

    // Pack 32× 6-bit values into 16× 12-bit values
    let packed1 = _mm256_maddubs_epi16(unpacked, _mm256_set1_epi16(0x0140));
    // Pack 16× 12-bit values into 8× 3-byte values
    let packed2 = _mm256_madd_epi16(packed1, _mm256_set1_epi32(0x00011000));
    // Pack 8× 3-byte values into 2× 12-byte values
    #[rustfmt::skip]
    let packed3 = _mm256_shuffle_epi8(packed2, dup_mm_setr_epu8([
           2,  1,  0,
           6,  5,  4,
          10,  9,  8,
          14, 13, 12,
          0xff, 0xff, 0xff, 0xff,
    ]));

    _mm_storeu_si128(
        output.as_mut_ptr() as _,
        _mm256_extracti128_si256(packed3, 0),
    );
    _mm_storeu_si128(
        output.as_mut_ptr().offset(12) as _,
        _mm256_extracti128_si256(packed3, 1),
    );
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

impl super::Decoder for Avx2 {
    type Block = [u8; 32];

    #[inline]
    fn decode_block(self, block: &mut Self::Block) -> super::BlockResult {
        // safe: `self` was given as a witness that the features are available
        unsafe { decode_block(block) }
    }

    #[inline(always)]
    fn zero_block() -> Self::Block {
        [b' '; 32]
    }
}

impl super::Packer for Avx2 {
    type Input = [u8; 32];
    const OUT_BUF_LEN: usize = 28;

    fn pack_block(self, input: &Self::Input, output: &mut [u8]) {
        // safe: `self` was given as a witness that the features are available
        unsafe { pack_block(input, output) }
    }
}
