/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::lut_align64::CacheLineLut;

static LUT_STANDARD: CacheLineLut = CacheLineLut(crate::misc::LUT_STANDARD);
static LUT_URLSAFE: CacheLineLut =
    CacheLineLut(*b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");

#[derive(Copy, Clone)]
pub(super) struct LutAlign64;

impl super::Encoder for LutAlign64 {
    type Block = [u8; 1];

    fn encode_block(self, block: &mut Self::Block, charset: crate::CharacterSet) {
        let lut = match charset {
            crate::Standard => &LUT_STANDARD,
            crate::UrlSafe => &LUT_URLSAFE,
        };
        block[0] = lut.0[block[0] as usize];
    }
}
