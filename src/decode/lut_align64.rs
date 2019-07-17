/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const INVALID_VALUE: u8 = 0x80;
const SPACE_VALUE: u8 = 0x40;

use crate::lut_align64::CacheLineLut;

static LUT1: CacheLineLut = CacheLineLut([
    INVALID_VALUE, // input 0 (0x0)
    INVALID_VALUE, // input 1 (0x1)
    INVALID_VALUE, // input 2 (0x2)
    INVALID_VALUE, // input 3 (0x3)
    INVALID_VALUE, // input 4 (0x4)
    INVALID_VALUE, // input 5 (0x5)
    INVALID_VALUE, // input 6 (0x6)
    INVALID_VALUE, // input 7 (0x7)
    INVALID_VALUE, // input 8 (0x8)
    SPACE_VALUE,   // input 9 (0x9)
    SPACE_VALUE,   // input 10 (0xA)
    INVALID_VALUE, // input 11 (0xB)
    SPACE_VALUE,   // input 12 (0xC)
    SPACE_VALUE,   // input 13 (0xD)
    INVALID_VALUE, // input 14 (0xE)
    INVALID_VALUE, // input 15 (0xF)
    INVALID_VALUE, // input 16 (0x10)
    INVALID_VALUE, // input 17 (0x11)
    INVALID_VALUE, // input 18 (0x12)
    INVALID_VALUE, // input 19 (0x13)
    INVALID_VALUE, // input 20 (0x14)
    INVALID_VALUE, // input 21 (0x15)
    INVALID_VALUE, // input 22 (0x16)
    INVALID_VALUE, // input 23 (0x17)
    INVALID_VALUE, // input 24 (0x18)
    INVALID_VALUE, // input 25 (0x19)
    INVALID_VALUE, // input 26 (0x1A)
    INVALID_VALUE, // input 27 (0x1B)
    INVALID_VALUE, // input 28 (0x1C)
    INVALID_VALUE, // input 29 (0x1D)
    INVALID_VALUE, // input 30 (0x1E)
    INVALID_VALUE, // input 31 (0x1F)
    SPACE_VALUE,   // input 32 (0x20)
    INVALID_VALUE, // input 33 (0x21)
    INVALID_VALUE, // input 34 (0x22)
    INVALID_VALUE, // input 35 (0x23)
    INVALID_VALUE, // input 36 (0x24)
    INVALID_VALUE, // input 37 (0x25)
    INVALID_VALUE, // input 38 (0x26)
    INVALID_VALUE, // input 39 (0x27)
    INVALID_VALUE, // input 40 (0x28)
    INVALID_VALUE, // input 41 (0x29)
    INVALID_VALUE, // input 42 (0x2A)
    62,            // input 43 (0x2B char '+') => 62 (0x3E)
    INVALID_VALUE, // input 44 (0x2C)
    62,            // input 45 (0x2D char '-') => 62 (0x3E)
    INVALID_VALUE, // input 46 (0x2E)
    63,            // input 47 (0x2F char '/') => 63 (0x3F)
    52,            // input 48 (0x30 char '0') => 52 (0x34)
    53,            // input 49 (0x31 char '1') => 53 (0x35)
    54,            // input 50 (0x32 char '2') => 54 (0x36)
    55,            // input 51 (0x33 char '3') => 55 (0x37)
    56,            // input 52 (0x34 char '4') => 56 (0x38)
    57,            // input 53 (0x35 char '5') => 57 (0x39)
    58,            // input 54 (0x36 char '6') => 58 (0x3A)
    59,            // input 55 (0x37 char '7') => 59 (0x3B)
    60,            // input 56 (0x38 char '8') => 60 (0x3C)
    61,            // input 57 (0x39 char '9') => 61 (0x3D)
    INVALID_VALUE, // input 58 (0x3A)
    INVALID_VALUE, // input 59 (0x3B)
    INVALID_VALUE, // input 60 (0x3C)
    INVALID_VALUE, // input 61 (0x3D)
    INVALID_VALUE, // input 62 (0x3E)
    INVALID_VALUE, // input 63 (0x3F)
]);

static LUT2: CacheLineLut = CacheLineLut([
    INVALID_VALUE, // input 64 (0x40)
    0,             // input 65 (0x41 char 'A') => 0 (0x0)
    1,             // input 66 (0x42 char 'B') => 1 (0x1)
    2,             // input 67 (0x43 char 'C') => 2 (0x2)
    3,             // input 68 (0x44 char 'D') => 3 (0x3)
    4,             // input 69 (0x45 char 'E') => 4 (0x4)
    5,             // input 70 (0x46 char 'F') => 5 (0x5)
    6,             // input 71 (0x47 char 'G') => 6 (0x6)
    7,             // input 72 (0x48 char 'H') => 7 (0x7)
    8,             // input 73 (0x49 char 'I') => 8 (0x8)
    9,             // input 74 (0x4A char 'J') => 9 (0x9)
    10,            // input 75 (0x4B char 'K') => 10 (0xA)
    11,            // input 76 (0x4C char 'L') => 11 (0xB)
    12,            // input 77 (0x4D char 'M') => 12 (0xC)
    13,            // input 78 (0x4E char 'N') => 13 (0xD)
    14,            // input 79 (0x4F char 'O') => 14 (0xE)
    15,            // input 80 (0x50 char 'P') => 15 (0xF)
    16,            // input 81 (0x51 char 'Q') => 16 (0x10)
    17,            // input 82 (0x52 char 'R') => 17 (0x11)
    18,            // input 83 (0x53 char 'S') => 18 (0x12)
    19,            // input 84 (0x54 char 'T') => 19 (0x13)
    20,            // input 85 (0x55 char 'U') => 20 (0x14)
    21,            // input 86 (0x56 char 'V') => 21 (0x15)
    22,            // input 87 (0x57 char 'W') => 22 (0x16)
    23,            // input 88 (0x58 char 'X') => 23 (0x17)
    24,            // input 89 (0x59 char 'Y') => 24 (0x18)
    25,            // input 90 (0x5A char 'Z') => 25 (0x19)
    INVALID_VALUE, // input 91 (0x5B)
    INVALID_VALUE, // input 92 (0x5C)
    INVALID_VALUE, // input 93 (0x5D)
    INVALID_VALUE, // input 94 (0x5E)
    63,            // input 95 (0x5F char '_') => 63 (0x3F)
    INVALID_VALUE, // input 96 (0x60)
    26,            // input 97 (0x61 char 'a') => 26 (0x1A)
    27,            // input 98 (0x62 char 'b') => 27 (0x1B)
    28,            // input 99 (0x63 char 'c') => 28 (0x1C)
    29,            // input 100 (0x64 char 'd') => 29 (0x1D)
    30,            // input 101 (0x65 char 'e') => 30 (0x1E)
    31,            // input 102 (0x66 char 'f') => 31 (0x1F)
    32,            // input 103 (0x67 char 'g') => 32 (0x20)
    33,            // input 104 (0x68 char 'h') => 33 (0x21)
    34,            // input 105 (0x69 char 'i') => 34 (0x22)
    35,            // input 106 (0x6A char 'j') => 35 (0x23)
    36,            // input 107 (0x6B char 'k') => 36 (0x24)
    37,            // input 108 (0x6C char 'l') => 37 (0x25)
    38,            // input 109 (0x6D char 'm') => 38 (0x26)
    39,            // input 110 (0x6E char 'n') => 39 (0x27)
    40,            // input 111 (0x6F char 'o') => 40 (0x28)
    41,            // input 112 (0x70 char 'p') => 41 (0x29)
    42,            // input 113 (0x71 char 'q') => 42 (0x2A)
    43,            // input 114 (0x72 char 'r') => 43 (0x2B)
    44,            // input 115 (0x73 char 's') => 44 (0x2C)
    45,            // input 116 (0x74 char 't') => 45 (0x2D)
    46,            // input 117 (0x75 char 'u') => 46 (0x2E)
    47,            // input 118 (0x76 char 'v') => 47 (0x2F)
    48,            // input 119 (0x77 char 'w') => 48 (0x30)
    49,            // input 120 (0x78 char 'x') => 49 (0x31)
    50,            // input 121 (0x79 char 'y') => 50 (0x32)
    51,            // input 122 (0x7A char 'z') => 51 (0x33)
    INVALID_VALUE, // input 123 (0x7B)
    INVALID_VALUE, // input 124 (0x7C)
    INVALID_VALUE, // input 125 (0x7D)
    INVALID_VALUE, // input 126 (0x7E)
    INVALID_VALUE, // input 127 (0x7F)
]);

fn decode64(b: u8) -> (u8, bool, bool) {
    let idx = (b % 64) as usize;

    /* This is basically what
     * ```
     * let mask = match b & 0xc0 {
     *     0x00 => 0,
     *     0x40 => 0xff,
     *     _ => unsafe { std::mem::MaybeUninit::uninit().assume_init() }
     * }
     * ```
     * compiles into.
     */
    let mask = (-(((b & 0xc0) == 0x40) as bool as i8)) as u8;

    let looked_up = ((!mask) & LUT1.0[idx]) | (mask & LUT2.0[idx]);
    (
        looked_up,
        ((b | looked_up) as i8).is_negative(),
        (looked_up & SPACE_VALUE) == SPACE_VALUE,
    )
}

#[derive(Copy, Clone)]
pub(super) struct LutAlign64;

impl super::Decoder for LutAlign64 {
    type Block = [u8; 1];

    #[inline]
    fn decode_block(self, block: &mut Self::Block) -> super::BlockResult {
        let (a, invalid, space) = decode64(block[0]);

        block[0] = a;

        super::BlockResult {
            out_length: if space | invalid { 0 } else { 1 },
            first_invalid: if invalid { Some(0) } else { None },
        }
    }

    #[inline(always)]
    fn zero_block() -> Self::Block {
        [b' '; 1]
    }
}
