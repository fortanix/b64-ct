/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod lut_align64;

use alloc::{string::String, vec::Vec};
use core::mem::size_of;

trait Encoder: Copy {
    type Block: AsRef<[u8]> + AsMut<[u8]> + Default;

    fn encode_block(self, block: &mut Self::Block, charset: crate::CharacterSet);
}

trait Unpacker: Copy {
    type Input: AsRef<[u8]> + AsMut<[u8]> + Default;
    type Output: AsRef<[u8]> + AsMut<[u8]> + Default;

    fn unpack_block(self, input: &Self::Input, output: &mut Self::Output);
}

#[derive(Copy, Clone)]
struct Simple;

impl Unpacker for Simple {
    type Input = [u8; 3];
    type Output = [u8; 4];

    fn unpack_block(self, input: &Self::Input, output: &mut Self::Output) {
        output[0] = input[0] >> 2;
        output[1] = ((input[0] & 0x03) << 4) | (input[1] >> 4);
        output[2] = ((input[1] & 0x0f) << 2) | (input[2] >> 6);
        output[3] = (input[2] & 0x3f) << 0;
    }
}

trait Lcm {
    type Array: AsRef<[u8]> + AsMut<[u8]> + Default;
}

const fn lcm(a: usize, b: usize) -> usize {
    // Euclidean algorithm
    const fn gcd(a: usize, b: usize) -> usize {
        if b == 0 {
            return a;
        }
        if b < a {
            return gcd(b, a);
        }
        gcd(a, b % a)
    }

    a * b / gcd(a, b)
}

/// Implement `Lcm` for `([u8; A], [u8; B])`, for each A and B in the cartesian
/// product of the input list with itself.
///
/// For example:
///
/// ```notest
/// impl_lcm_array!([32 4 1]);
/// ```
///
/// This would implement for (A,B) in { (32,32) (32,4) (32,1) (4,32) (4,4) (4,1) (1,32) (1,4) (1,1) }.
macro_rules! impl_lcm_array {
    ([$first:literal $($rest:literal)*]) => {
        $(
            const _: () = const {
                if $rest >= $first {
                    panic!("The array sizes in the impl_lcm_array! invocation must appear in monotonically decreasing order");
                }
            };

            impl_lcm_array!(@impl $first $rest);
            impl_lcm_array!(@impl $rest $first);
        )*
        impl_lcm_array!(@impl $first $first);

        impl_lcm_array!([$($rest)*]);
    };
    ([]) => {};
    (@impl $a:literal $b:literal) => {
        impl Lcm for ([u8; $a], [u8; $b]) {
            type Array = [u8; lcm($a, $b)];
        }
    }
}

impl_lcm_array!([32 4 1]);

trait SplitFrom<T>: Sized {
    fn split_from(from: &mut T) -> &mut [Self];
}

impl<const M: usize, const N: usize> SplitFrom<[u8; N]> for [u8; M] {
    /// Turn an array of size N into an slice of arrays with a total size of A * M.
    /// Only works if M is an integer divisor of N.
    fn split_from(from: &mut [u8; N]) -> &mut [Self] {
        // Safety:
        //
        // The lifetime, size and alignment of the output is exactly the same
        // as the input
        unsafe {
            const {
                assert!(M <= N);
                assert!(N % M == 0);
                assert!(core::mem::align_of::<Self>() == core::mem::align_of::<u8>());
                assert!(N / M * size_of::<Self>() == size_of::<[u8; N]>());
            }
            core::slice::from_raw_parts_mut(from.as_mut_ptr() as _, N / M)
        }
    }
}

trait TakePrefix: Sized {
    fn take_prefix(&mut self, mid: usize) -> Self;
}

impl<'a, T: 'a> TakePrefix for &'a [T] {
    fn take_prefix(&mut self, mid: usize) -> Self {
        let prefix = &self[..mid];
        *self = &self[mid..];
        prefix
    }
}

impl crate::Newline {
    fn append_to(self, buf: &mut Vec<u8>) {
        if let crate::Newline::CRLF = self {
            buf.push(b'\r');
        }
        buf.push(b'\n');
    }
}

fn encode64<E: Encoder, U: Unpacker>(
    input: &[u8],
    config: crate::Config,
    encoder: E,
    unpacker: U,
) -> String
where
    (U::Output, E::Block): Lcm,
    U::Output: SplitFrom<<(U::Output, E::Block) as Lcm>::Array>,
    E::Block: SplitFrom<<(U::Output, E::Block) as Lcm>::Array>,
{
    let mut len = crate::misc::div_roundup(input.len(), 3) * 4;
    let mut next_nl = config.line_length;
    if let Some(line_length) = config.line_length {
        let nl_len = match config.newline {
            crate::Newline::LF => 1,
            crate::Newline::CRLF => 2,
        };
        len = crate::misc::div_roundup(len, line_length) * (line_length + nl_len);
    }
    let mut output = Vec::with_capacity(len);

    let mut buffer = <(U::Output, E::Block) as Lcm>::Array::default();

    let mut input_iter = input.chunks(size_of::<U::Input>());
    while input_iter.len() > 0 {
        let mut input_len = 0;
        for chunk in <U::Output>::split_from(&mut buffer) {
            let mut input_block = U::Input::default();
            if let Some(input_next) = input_iter.next() {
                input_len += input_next.len();
                input_block.as_mut()[..input_next.len()].copy_from_slice(input_next);
            }
            unpacker.unpack_block(&input_block, chunk);
        }
        for chunk in <E::Block>::split_from(&mut buffer) {
            encoder.encode_block(chunk, config.char_set);
        }

        let mut buffer = &buffer.as_ref()[..crate::misc::div_roundup(input_len * 4, 3)];

        if let Some(mut nl_index) = next_nl {
            while (output.len() + buffer.len()) > nl_index {
                let line = buffer.take_prefix(nl_index - output.len());
                output.extend_from_slice(&line);
                config.newline.append_to(&mut output);
                nl_index = output.len() + config.line_length.unwrap();
            }
            next_nl = Some(nl_index);
        }

        output.extend_from_slice(buffer);
    }

    if config.pad {
        if let Some(mut nl_index) = next_nl {
            let trailer_length = match input.len() % 3 {
                1 => 2,
                2 => 1,
                _ => 0,
            };
            for _ in 0..trailer_length {
                if output.len() == nl_index {
                    config.newline.append_to(&mut output);
                    nl_index = output.len() + config.line_length.unwrap();
                }
                output.push(b'=');
            }
        } else if output.len() != len {
            output.resize(len, b'=');
        }
    }

    String::from_utf8(output).unwrap()
}

pub(super) fn encode64_arch(input: &[u8], config: crate::Config) -> String {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        if is_x86_feature_detected!("avx2") {
            let avx2 = avx2::Avx2::new();
            return encode64(input, config, avx2, avx2);
        }
    }
    encode64(input, config, lut_align64::LutAlign64, Simple)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Config, Newline, STANDARD, URL_SAFE};

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub(super) fn test_avx2() -> avx2::Avx2 {
        unsafe { avx2::Avx2::new() }
    }

    generate_tests![
        encoders<E>: {
            lut_align64, lut_align64::LutAlign64;
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] avx2, test_avx2();
        },
        unpackers<U>: {
            simple, Simple;
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] avx2, test_avx2();
        },
        tests: {
            encode,
        },
    ];

    fn encode<E: Encoder, U: Unpacker>(encoder: E, unpacker: U)
    where
        (U::Output, E::Block): Lcm,
        U::Output: SplitFrom<<(U::Output, E::Block) as Lcm>::Array>,
        E::Block: SplitFrom<<(U::Output, E::Block) as Lcm>::Array>,
    {
        static ENCODE_TESTS: &[(&[u8], Config, &str)] = &[
            // basic tests (from rustc-serialize)
            (b"", STANDARD, ""),
            (b"f", STANDARD, "Zg=="),
            (b"fo", STANDARD, "Zm8="),
            (b"foo", STANDARD, "Zm9v"),
            (b"foob", STANDARD, "Zm9vYg=="),
            (b"fooba", STANDARD, "Zm9vYmE="),
            (b"foobar", STANDARD, "Zm9vYmFy"),
            // with crlf break (from rustc-serialize)
            (b"foobar", Config {line_length: Some(4), ..STANDARD}, "Zm9v\r\nYmFy"),
            // with lf break (from rustc-serialize)
            (b"foobar", Config {line_length: Some(4), newline: Newline::LF, ..STANDARD}, "Zm9v\nYmFy"),
            // without padding (from rustc-serialize)
            (b"f", Config {pad: false, ..STANDARD}, "Zg"),
            (b"fo", Config {pad: false, ..STANDARD}, "Zm8"),
            // URL safe (from rustc-serialize)
            (&[251, 255], URL_SAFE, "-_8"),
            (&[251, 255], STANDARD, "+/8="),

            // new tests
            (b"f", Config {line_length: Some(1), ..STANDARD}, "Z\r\ng\r\n=\r\n="),
            (b"fo", Config {line_length: Some(1), ..STANDARD}, "Z\r\nm\r\n8\r\n="),
            (b"foob", Config {line_length: Some(4), ..STANDARD}, "Zm9v\r\nYg=="),
            (b"foob", Config {line_length: Some(5), ..STANDARD}, "Zm9vY\r\ng=="),
            (b"foob", Config {line_length: Some(6), ..STANDARD}, "Zm9vYg\r\n=="),
            (b"foob", Config {line_length: Some(7), ..STANDARD}, "Zm9vYg=\r\n="),
            (b"foob", Config {line_length: Some(8), ..STANDARD}, "Zm9vYg=="),
            (b"foobfoo", Config {line_length: Some(3), ..STANDARD}, "Zm9\r\nvYm\r\nZvb\r\nw=="),
            (b"foobfoo", Config {line_length: Some(4), ..STANDARD}, "Zm9v\r\nYmZv\r\nbw=="),
            (b"foobfoo", Config {line_length: Some(5), ..STANDARD}, "Zm9vY\r\nmZvbw\r\n=="),
            (b"\x00\x10\x83\x10\x51\x87\x20\x92\x8b\x30\xd3\x8f\x41\x14\x93\x51\x55\x97\x61\x96\x9b\x71\xd7\x9f", STANDARD, "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef"),
        ];

        for (input, config, expected) in ENCODE_TESTS {
            let output = encode64(input, *config, encoder, unpacker);
            if &output != expected {
                panic!(
                    "Test failed. Expected specific output. \n\nInput: {:02x?}\nOutput: {}\nExpected output:{}\n\n",
                    input,
                    output,
                    expected
                );
            }
        }
    }
}

#[cfg(all(test, feature = "nightly"))]
mod benches {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    use super::tests::test_avx2;
    use super::*;

    use test::Bencher;

    use rand::{thread_rng, RngCore};

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[bench]
    fn avx2_1mb(b: &mut Bencher) {
        let mut input = std::vec![0; 1024*1024];
        thread_rng().fill_bytes(&mut input);
        b.iter(|| {
            let ret = encode64(&input, crate::STANDARD, test_avx2(), test_avx2());
            std::hint::black_box(ret);
        });
    }

    #[bench]
    fn lut_align64_1mb(b: &mut Bencher) {
        let mut input = std::vec![0; 1024*1024];
        thread_rng().fill_bytes(&mut input);
        b.iter(|| {
            let ret = encode64(&input, crate::STANDARD, lut_align64::LutAlign64, Simple);
            std::hint::black_box(ret);
        });
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[bench]
    fn avx2_1kb(b: &mut Bencher) {
        let mut input = std::vec![0; 1024];
        thread_rng().fill_bytes(&mut input);
        b.iter(|| {
            let ret = encode64(&input, crate::STANDARD, test_avx2(), test_avx2());
            std::hint::black_box(ret);
        });
    }

    #[bench]
    fn lut_align64_1kb(b: &mut Bencher) {
        let mut input = std::vec![0; 1024];
        thread_rng().fill_bytes(&mut input);
        b.iter(|| {
            let ret = encode64(&input, crate::STANDARD, lut_align64::LutAlign64, Simple);
            std::hint::black_box(ret);
        });
    }
}
