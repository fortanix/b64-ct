/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! is_x86_feature_detected {
    ($feat:literal) => {
        cfg!(target_feature = $feat)
    };
}

pub(crate) const LUT_STANDARD: [u8; 64] =
    *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline(always)]
pub(crate) fn div_roundup(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}
