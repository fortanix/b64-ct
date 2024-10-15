/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This crate provides an implementation of Base64 encoding/decoding that is
//! designed to be resistant against software side-channel attacks (such as
//! timing & cache attacks), see below for details. On certain platforms it
//! also uses SIMD making it very fast. This makes it suitable for e.g.
//! decoding cryptographic private keys in PEM format.
//!
//! The API is very similar to the base64 implementation in the old
//! rustc-serialize crate, making it easy to use in existing projects.
//!
//! # Resistance against Software Side-Channel Attacks
//!
//! An indistinguishable-time (colloquially: constant-time) implementation of
//! an algorithm has a runtime that's independent of the data being processed.
//! This indistinguishability is usually based on the control flow of the
//! program as well as its memory access pattern. In that case,
//! indistinguishability may be achieved by making sure the control flow and
//! memory access pattern don't depend on the data. Other factors, such as
//! instruction cycle count may also be consequential.
//!
//! See the [BearSSL page on constant-time cryptography] for more information.
//!
//! The runtime of the implementations in this crate is intended to be
//! dependent only on whitespace and the length of the valid data, not the data
//! itself.
//!
//! [BearSSL page on constant-time cryptography]: https://bearssl.org/constanttime.html
//!
//! # Implementation
//!
//! Depending on the runtime CPU architecture, this crate uses different
//! implementations with different security properties.
//!
//! * x86 with AVX2: All lookup tables are implemented with SIMD
//!   instructions. No secret-dependent memory accceses.
//! * Other platforms: Lookups are limited to 64-byte aligned lookup tables. On
//!   platforms with 64-byte cache lines this may be sufficient to prevent
//!   certain cache side-channel attacks. However, it's known that this is [not
//!   sufficient for all platforms].
//!
//! [not sufficient on some platforms]: https://ts.data61.csiro.au/projects/TS/cachebleed/

#![no_std]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
#[allow(unused_imports)]
#[macro_use]
extern crate std;
#[cfg(all(test, feature = "nightly"))]
extern crate test;

#[cfg(test)]
#[macro_use]
mod test_support;

#[macro_use]
mod misc;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod lut_align64;

mod decode;
mod encode;

use alloc::{string::String, vec::Vec};

pub use self::CharacterSet::*;

/// Available encoding character sets
#[derive(Clone, Copy, Debug)]
pub enum CharacterSet {
    /// The standard character set (uses `+` and `/`)
    Standard,
    /// The URL safe character set (uses `-` and `_`)
    UrlSafe,
}

/// Available newline types
#[derive(Clone, Copy, Debug)]
pub enum Newline {
    /// A linefeed (i.e. Unix-style newline)
    LF,
    /// A carriage return and a linefeed (i.e. Windows-style newline)
    CRLF,
}

/// Contains configuration parameters for `to_base64`.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Character set to use
    pub char_set: CharacterSet,
    /// Newline to use
    pub newline: Newline,
    /// True to pad output with `=` characters
    pub pad: bool,
    /// `Some(len)` to wrap lines at `len`, `None` to disable line wrapping
    pub line_length: Option<usize>,
}

/// Configuration for RFC 4648 standard base64 encoding
pub static STANDARD: Config = Config {
    char_set: Standard,
    newline: Newline::CRLF,
    pad: true,
    line_length: None,
};

/// Configuration for RFC 4648 base64url encoding
pub static URL_SAFE: Config = Config {
    char_set: UrlSafe,
    newline: Newline::CRLF,
    pad: false,
    line_length: None,
};

/// Configuration for RFC 2045 MIME base64 encoding
pub static MIME: Config = Config {
    char_set: Standard,
    newline: Newline::CRLF,
    pad: true,
    line_length: Some(76),
};

/// A trait for converting a value to base64 encoding.
pub trait ToBase64 {
    /// Converts the value of `self` to a base64 value following the specified
    /// format configuration, returning the owned string.
    fn to_base64(&self, config: Config) -> String;
}

impl ToBase64 for [u8] {
    /// Turn a vector of `u8` bytes into a base64 string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use b64_ct::{ToBase64, STANDARD};
    ///
    /// fn main () {
    ///     let str = [52,32].to_base64(STANDARD);
    ///     println!("base 64 output: {:?}", str);
    /// }
    /// ```
    fn to_base64(&self, config: Config) -> String {
        encode::encode64_arch(self, config)
    }
}

impl<'a, T: ?Sized + ToBase64> ToBase64 for &'a T {
    fn to_base64(&self, config: Config) -> String {
        (**self).to_base64(config)
    }
}

#[doc(inline)]
pub use decode::Error as FromBase64Error;

/// A trait for converting from base64 encoded values.
pub trait FromBase64 {
    /// Converts the value of `self`, interpreted as base64 encoded data, into
    /// an owned vector of bytes, returning the vector.
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error>;
}

impl FromBase64 for str {
    /// Convert any base64 encoded string (literal, `@`, `&`, or `~`)
    /// to the byte values it encodes.
    ///
    /// You can use the `String::from_utf8` function to turn a `Vec<u8>` into a
    /// string with characters corresponding to those values.
    ///
    /// # Example
    ///
    /// This converts a string literal to base64 and back.
    ///
    /// ```rust
    /// use b64_ct::{ToBase64, FromBase64, STANDARD};
    ///
    /// fn main () {
    ///     let hello_str = b"Hello, World".to_base64(STANDARD);
    ///     println!("base64 output: {}", hello_str);
    ///     let res = hello_str.from_base64();
    ///     if res.is_ok() {
    ///       let opt_bytes = String::from_utf8(res.unwrap());
    ///       if opt_bytes.is_ok() {
    ///         println!("decoded from base64: {:?}", opt_bytes.unwrap());
    ///       }
    ///     }
    /// }
    /// ```
    #[inline]
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error> {
        self.as_bytes().from_base64()
    }
}

impl FromBase64 for [u8] {
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error> {
        decode::decode64_arch(self)
    }
}

impl<'a, T: ?Sized + FromBase64> FromBase64 for &'a T {
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error> {
        (**self).from_base64()
    }
}
