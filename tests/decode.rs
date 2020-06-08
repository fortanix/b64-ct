use b64_ct::*;

const BASE64_PEM_WRAP: usize = 64;

static BASE64_PEM: b64_ct::Config = b64_ct::Config {
    char_set: b64_ct::CharacterSet::Standard,
    newline: b64_ct::Newline::LF,
    pad: true,
    line_length: Some(BASE64_PEM_WRAP),
};

#[test]
fn test_wrapping_base64() {
    let mut v: Vec<u8> = vec![];
    let bytes_per_line = BASE64_PEM_WRAP * 3 / 4;
    for _i in 0..2*bytes_per_line {
        let encoded = v.to_base64(BASE64_PEM);
        let decoded = encoded.from_base64().unwrap();
        assert_eq!(v, decoded);
        v.push(0);
    }
}

#[test]
fn test_wrapping_base64_random() {
    let mut v: Vec<u8> = vec![];
    for _i in 0..1000 {
        let encoded = v.to_base64(BASE64_PEM);
        let decoded = encoded.from_base64().unwrap();
        assert_eq!(v, decoded);
        v.push(rand::random::<u8>());
    }
}

#[test]
fn test_constant_strings() {
    assert_eq!("".from_base64().unwrap(), vec![]);
    assert_eq!("Zg==".from_base64().unwrap(), "f".as_bytes());
    assert_eq!("Zg".from_base64().unwrap(), "f".as_bytes());
    assert_eq!("Zg".from_base64().unwrap(), "f".as_bytes());
    assert_eq!("Zm8=".from_base64().unwrap(), "fo".as_bytes());
    assert_eq!("Zm8".from_base64().unwrap(), "fo".as_bytes());
    assert_eq!("Zm9y".from_base64().unwrap(), "for".as_bytes());
    assert_eq!("Zm9ydA==".from_base64().unwrap(), "fort".as_bytes());
    assert_eq!("Zm9ydA".from_base64().unwrap(), "fort".as_bytes());
    assert_eq!("Zm9v".from_base64().unwrap(), "foo".as_bytes());
    assert_eq!("Zm9vYg==".from_base64().unwrap(), "foob".as_bytes());
    assert_eq!("Zm9vYg".from_base64().unwrap(), "foob".as_bytes());
    assert_eq!("Zm9vYmE=".from_base64().unwrap(), "fooba".as_bytes());
    assert_eq!("Zm9vYmE".from_base64().unwrap(), "fooba".as_bytes());
    assert_eq!("Zm9vYmFy".from_base64().unwrap(), "foobar".as_bytes());

    "YWx\0pY2U==".from_base64().unwrap_err();

}

#[test]
fn decode_1_pad_byte_in_fast_loop_then_extra_padding_chunk_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2U=====");

        // since the first 8 bytes are handled in stage 1 or 2, the padding is detected as a
        // generic invalid byte, not specifcally a padding issue.
        // Could argue that the *next* padding byte (in the next quad) is technically the first
        // erroneous one, but reporting that accurately is more complex and probably nobody cares
        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_2_pad_bytes_in_leftovers_then_extra_padding_chunk_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2UABB====");

        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_valid_bytes_after_padding_in_leftovers_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2UABB=B");

        // 4 bytes after last 8-byte chunk, so it's decoded by stage 4.
        // First (and only) padding byte is invalid.
        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_absurd_pad_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("==Y=Wx===pY=2U=====");

        // Plenty of remaining bytes, so handled by stage 1 or 2.
        // first padding byte
        s.from_base64().unwrap_err();
    }
}
#[test]
fn decode_extra_padding_after_1_pad_bytes_in_trailing_quad_returns_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("EEE===");

        // handled by stage 1, 2, or 4 depending on length
        // first padding byte -- which would be legal if it was the only padding
        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_extra_padding_after_2_pad_bytes_in_trailing_quad_2_returns_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("EE====");

        // handled by stage 1, 2, or 4 depending on length
        // first padding byte -- which would be legal if it was by itself
        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_start_quad_with_padding_returns_error() {
    for num_quads in 0..25 {
        // add enough padding to ensure that we'll hit all 4 stages at the different lengths
        for pad_bytes in 1..32 {
            let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
            let padding: String = std::iter::repeat("=").take(pad_bytes).collect();
            s.push_str(&padding);

            s.from_base64().unwrap_err();
        }
    }
}

#[test]
fn decode_padding_followed_by_non_padding_returns_error() {
    for num_quads in 0..25 {
        for pad_bytes in 0..31 {
            let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
            let padding: String = std::iter::repeat("=").take(pad_bytes).collect();
            s.push_str(&padding);
            s.push_str("E");

            s.from_base64().unwrap_err();
        }
    }
}

#[test]
fn decode_one_char_in_quad_with_padding_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("E=");

        s.from_base64().unwrap_err();

        // more padding doesn't change the error
        s.push_str("=");
        s.from_base64().unwrap_err();

        s.push_str("=");
        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_one_char_in_quad_without_padding_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push('E');

        s.from_base64().unwrap_err();
    }
}

#[test]
fn decode_reject_invalid_bytes_with_correct_error() {
    for length in 1..100 {
        for index in 0_usize..length {
            for invalid_byte in "\x0B\x00%*.".bytes() {
                let prefix: String = std::iter::repeat("A").take(index).collect();
                let suffix: String = std::iter::repeat("B").take(length - index - 1).collect();

                let input = prefix + &String::from_utf8(vec![invalid_byte]).unwrap() + &suffix;
                assert_eq!(
                    length,
                    input.len(),
                    "length {} error position {}",
                    length,
                    index
                );

                input.from_base64().unwrap_err();
            }
        }
    }
}

