/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod avx2;
mod lut_align64;

use alloc::vec::Vec;
use core::cmp;
use core::fmt;

#[must_use]
struct BlockResult {
    out_length: u8,
    first_invalid: Option<u8>,
}

/// Errors that can occur when decoding a base64 encoded string
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The input had an invalid length.
    InvalidLength,
    /// A trailer was found, but it wasn't the right length.
    InvalidTrailer,
    /// The input contained a character (at the given index) not part of the
    /// base64 format.
    InvalidCharacter(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

trait Decoder: Copy {
    type Block: AsRef<[u8]> + AsMut<[u8]>;

    fn decode_block(self, block: &mut Self::Block) -> BlockResult;
    fn zero_block() -> Self::Block;
}

trait Packer: Copy {
    type Input: AsRef<[u8]> + AsMut<[u8]> + Default;
    const OUT_BUF_LEN: usize;

    /// The caller should pass `output` as a slice with length `OUT_BUF_LEN`.
    fn pack_block(self, input: &Self::Input, output: &mut [u8]);
}

#[derive(Copy, Clone)]
struct Simple;

impl Packer for Simple {
    type Input = [u8; 4];
    const OUT_BUF_LEN: usize = 3;

    #[inline]
    fn pack_block(self, input: &Self::Input, output: &mut [u8]) {
        output[0] = (input[0] << 2) | (input[1] >> 4);
        output[1] = (input[1] << 4) | (input[2] >> 2);
        output[2] = (input[2] << 6) | (input[3] >> 0);
    }
}

struct PackState<P: Packer> {
    packer: P,
    cache: P::Input,
    pos: usize,
}

impl<P: Packer> PackState<P> {
    fn extend(&mut self, mut input: &[u8], out: &mut Vec<u8>) {
        while !input.is_empty() {
            let (_, cache_end) = self.cache.as_mut().split_at_mut(self.pos);
            let (input_start, input_rest) = input.split_at(cmp::min(input.len(), cache_end.len()));
            input = input_rest;
            cache_end[..input_start.len()].copy_from_slice(input_start);
            if input_start.len() != cache_end.len() {
                self.pos += input_start.len();
            } else {
                let out_start = out.len();
                out.resize(out_start + P::OUT_BUF_LEN, 0);
                self.packer.pack_block(&self.cache, &mut out[out_start..]);
                out.truncate(out_start + (core::mem::size_of::<P::Input>() / 4 * 3));
                self.pos = 0;
            }
        }
    }

    fn flush(&mut self, out: &mut Vec<u8>, trailer_length: Option<usize>) -> Result<(), Error> {
        if self.pos % 4 == 1 {
            return Err(Error::InvalidLength);
        }

        if let Some(trailer_length) = trailer_length {
            if (self.pos + trailer_length) % 4 != 0 {
                return Err(Error::InvalidTrailer);
            }
        }

        self.cache.as_mut()[self.pos] = 0;
        let out_start = out.len();
        out.resize(out.len() + P::OUT_BUF_LEN, 0);
        self.packer.pack_block(&self.cache, &mut out[out_start..]);
        out.truncate(out_start + (self.pos * 3 / 4));
        Ok(())
    }
}

fn decode64<D: Decoder, P: Packer>(input: &[u8], decoder: D, packer: P) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let p_in_len = core::mem::size_of::<P::Input>();
    let p_out_len = p_in_len / 4 * 3;
    let cap =
        crate::misc::div_roundup(input.len(), p_in_len) * p_out_len - p_out_len + P::OUT_BUF_LEN;
    let mut out = Vec::with_capacity(cap);

    let mut packer = PackState::<P> {
        packer,
        cache: P::Input::default(),
        pos: 0,
    };

    let mut trailer_length = None;
    for (chunk, chunk_start) in input
        .chunks(core::mem::size_of::<D::Block>())
        .zip((0..).step_by(core::mem::size_of::<D::Block>()))
    {
        let mut block = D::zero_block();
        block.as_mut()[..chunk.len()].copy_from_slice(chunk);
        let result = decoder.decode_block(&mut block);

        if let Some(idx) = result.first_invalid {
            let idx = idx as usize;
            if input[chunk_start + idx] == b'=' {
                let rest_start = chunk_start + idx + 1;
                let rest = &input[rest_start..];
                let mut iter = rest
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| !c.is_ascii_whitespace());
                trailer_length = match (iter.next(), iter.next()) {
                    (None, _) => Some(1),
                    (Some((_, b'=')), None) => Some(2),
                    (Some((_, b'=')), Some((i, _))) | (Some((i, _)), _) => {
                        return Err(Error::InvalidCharacter(rest_start + i))
                    }
                };
            } else {
                return Err(Error::InvalidCharacter(chunk_start + idx));
            }
        }

        packer.extend(&block.as_ref()[..(result.out_length as _)], &mut out);

        if trailer_length.is_some() {
            break;
        }
    }

    packer.flush(&mut out, trailer_length)?;

    Ok(out)
}

pub(super) fn decode64_arch(input: &[u8]) -> Result<Vec<u8>, Error> {
    unsafe {
        if is_x86_feature_detected!("avx2")
            && is_x86_feature_detected!("bmi1")
            && is_x86_feature_detected!("sse4.2")
            && is_x86_feature_detected!("popcnt")
        {
            let avx2 = avx2::Avx2::new();
            return decode64(input, avx2, avx2);
        }
    }
    decode64(input, lut_align64::LutAlign64, Simple)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::rand_base64_size;
    use crate::{ToBase64};

    pub(super) fn test_avx2() -> avx2::Avx2 {
        unsafe { avx2::Avx2::new() }
    }

    generate_tests![
        decoders<D>: {
            avx2, test_avx2();
            lut_align64, lut_align64::LutAlign64;
        },
        packers<P>: {
            avx2, test_avx2();
            simple, Simple;
        },
        tests: {
            decode,
            decode_equivalency,
            decode_error,
            cmp_rand_1kb,
            whitespace_skipped,
            all_bytes,
            wrapping_base64,
        },
    ];

    fn decode<D: Decoder, P: Packer>(decoder: D, packer: P) {
        static DECODE_TESTS: &[(&[u8], &[u8])] = &[
            // basic tests (from rustc-serialize)
            (b"", b""),
            (b"Zg==", b"f"),
            (b"Zm8=", b"fo"),
            (b"Zm9v", b"foo"),
            (b"Zm9vYg==", b"foob"),
            (b"Zm9vYmE=", b"fooba"),
            (b"Zm9vYmFy", b"foobar"),
            // with newlines (from rustc-serialize)
            (b"Zm9v\r\nYmFy", b"foobar"),
            (b"Zm9vYg==\r\n", b"foob"),
            (b"Zm9v\nYmFy", b"foobar"),
            (b"Zm9vYg==\n", b"foob"),
            // white space in trailer
            (b"Zm9vYg  =  =  ", b"foob"),
        ];

        for (input, expected) in DECODE_TESTS {
            let output = decode64(input, decoder, packer).unwrap();
            if &output != expected {
                panic!(
                    "Test failed. Expected specific output. \n\nInput: {}\nOutput: {:02x?}\nExpected output:{:02x?}\n\n",
                    std::str::from_utf8(input).unwrap(),
                    output,
                    expected
                );
            }
        }
    }

    fn decode_equivalency<D: Decoder, P: Packer>(decoder: D, packer: P) {
        static DECODE_EQUIVALENCY_TESTS: &[(&[u8], &[u8])] = &[
            // url safe test (from rustc-serialize)
            (b"-_8", b"+/8="),
        ];

        for (input1, input2) in DECODE_EQUIVALENCY_TESTS {
            let output1 = decode64(input1, decoder, packer).unwrap();
            let output2 = decode64(input2, decoder, packer).unwrap();
            if output1 != output2 {
                panic!(
                    "Test failed. Expected same output.\n\nInput 1: {}\nInput 2: {}\nOutput 1: {:02x?}\nOutput 2:{:02x?}\n\n",
                    std::str::from_utf8(input1).unwrap(),
                    std::str::from_utf8(input2).unwrap(),
                    output1,
                    output2
                );
            }
        }
    }

    fn decode_error<D: Decoder, P: Packer>(decoder: D, packer: P) {
        #[rustfmt::skip]
        static DECODE_ERROR_TESTS: &[&[u8]] = &[
            // invalid chars (from rustc-serialize)
            b"Zm$=",
            b"Zg==$",
            // invalid padding (from rustc-serialize)
            b"Z===",
        ];

        for input in DECODE_ERROR_TESTS {
            if decode64(input, decoder, packer).is_ok() {
                panic!(
                    "Test failed. Expected error.\n\nInput: {}\n\n",
                    std::str::from_utf8(input).unwrap(),
                );
            }
        }
    }

    fn cmp_rand_1kb<D: Decoder, P: Packer>(decoder: D, packer: P) {
        let input = rand_base64_size(1024);

        let output1 = decode64(&input, decoder, packer).unwrap();
        let output2 = decode64(&input, lut_align64::LutAlign64, Simple).unwrap();
        if output1 != output2 {
            panic!(
                "Test failed. Expected same output.\n\nInput: {}\nOutput 1: {:02x?}\nOutput 2:{:02x?}\n\n",
                std::str::from_utf8(&input).unwrap(),
                output1,
                output2
            );
        }
    }

    fn whitespace_skipped<D: Decoder, P: Packer>(decoder: D, packer: P) {
        let input1 = rand_base64_size(32);
        use core::iter::once;
        let input2 = input1
            .iter()
            .flat_map(|&c| once(c).chain(once(b' ')))
            .collect::<Vec<_>>();

        let output1 = decode64(&input1, decoder, packer).unwrap();
        let output2 = decode64(&input2, decoder, packer).unwrap();
        if output1 != output2 {
            panic!(
                "Test failed. Expected same output.\n\nInput 1: {}\nInput 2: {}\nOutput 1: {:02x?}\nOutput 2:{:02x?}\n\n",
                std::str::from_utf8(&input1).unwrap(),
                std::str::from_utf8(&input2).unwrap(),
                output1,
                output2
            );
        }
    }

    fn all_bytes<D: Decoder, P: Packer>(decoder: D, packer: P) {
        let mut set = std::vec![Err(()); 256];
        for (i, &b) in crate::misc::LUT_STANDARD.iter().enumerate() {
            set[b as usize] = Ok(Some(i as u8));
        }
        // add URL-safe set
        set[b'-' as usize] = Ok(Some(62));
        set[b'_' as usize] = Ok(Some(63));
        // add whitespace
        set[b' ' as usize] = Ok(None);
        set[b'\n' as usize] = Ok(None);
        set[b'\t' as usize] = Ok(None);
        set[b'\r' as usize] = Ok(None);
        set[0x0c] = Ok(None);

        for (i, &expected) in set.iter().enumerate() {
            let output = match decode64(&[i as u8, i as u8], decoder, packer)
                .as_ref()
                .map(|v| &v[..])
            {
                Ok(&[]) => Ok(None),
                Ok(&[v]) => Ok(Some(v >> 2)),
                Ok(_) => panic!("Result is more than 1 byte long"),
                Err(_) => Err(()),
            };
            assert_eq!(output, expected);
        }
    }

    fn wrapping_base64<D: Decoder, P: Packer>(decoder: D, packer: P) {
        const BASE64_PEM_WRAP: usize = 64;

        static BASE64_PEM: crate::Config = crate::Config {
            char_set: crate::CharacterSet::Standard,
            newline: crate::Newline::LF,
            pad: true,
            line_length: Some(BASE64_PEM_WRAP),
        };

        let mut v: Vec<u8> = vec![];
        let bytes_per_line = BASE64_PEM_WRAP * 3 / 4;
        for _i in 0..2*bytes_per_line {
            let encoded = v.to_base64(BASE64_PEM);
            let decoded = decode64(encoded.as_bytes(), decoder, packer).unwrap();
            assert_eq!(v, decoded);
            v.push(0);
        }

        v = vec![];
        for _i in 0..1000 {
            let encoded = v.to_base64(BASE64_PEM);
            let decoded = decode64(encoded.as_bytes(), decoder, packer).unwrap();
            assert_eq!(v, decoded);
            v.push(rand::random::<u8>());
        }
    }

    #[test]
    fn display_errors() {
        println!("Invalid length is {}", Error::InvalidLength);
        println!("Invalid trailer is {}", Error::InvalidTrailer);
        println!("Invalid character is {}", Error::InvalidCharacter(0));
    }
}

#[cfg(all(test, feature = "nightly"))]
mod benches {
    use super::{tests::test_avx2, *};

    use test::Bencher;

    use crate::test_support::rand_base64_size;

    #[bench]
    fn avx2_1mb(b: &mut Bencher) {
        let input = rand_base64_size(1024 * 1024);
        b.iter(|| {
            let ret = decode64(&input, test_avx2(), test_avx2()).unwrap();
            std::hint::black_box(ret);
        });
    }

    #[bench]
    fn lut_align64_1mb(b: &mut Bencher) {
        let input = rand_base64_size(1024 * 1024);
        b.iter(|| {
            let ret = decode64(&input, lut_align64::LutAlign64, Simple).unwrap();
            std::hint::black_box(ret);
        });
    }

    #[bench]
    fn avx2_1kb(b: &mut Bencher) {
        let input = rand_base64_size(1024);
        b.iter(|| {
            let ret = decode64(&input, test_avx2(), test_avx2()).unwrap();
            std::hint::black_box(ret);
        });
    }

    #[bench]
    fn lut_align64_1kb(b: &mut Bencher) {
        let input = rand_base64_size(1024);
        b.iter(|| {
            let ret = decode64(&input, lut_align64::LutAlign64, Simple).unwrap();
            std::hint::black_box(ret);
        });
    }
}
