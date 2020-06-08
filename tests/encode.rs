use b64_ct::*;

const BASE64_PEM_WRAP: usize = 100000;

static BASE64_PEM: b64_ct::Config = b64_ct::Config {
    char_set: b64_ct::CharacterSet::Standard,
    newline: b64_ct::Newline::LF,
    pad: true,
    line_length: Some(BASE64_PEM_WRAP),
};

#[test]
fn test_constant_strings() {
    assert_eq!("", "".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zg==", "f".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm8=", "fo".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9y", "for".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9ydA==", "fort".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9v", "foo".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9vYg==", "foob".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9vYmE=", "fooba".as_bytes().to_base64(BASE64_PEM));
    assert_eq!("Zm9vYmFy", "foobar".as_bytes().to_base64(BASE64_PEM));
}

#[test]
fn encode_all_ascii() {
    let mut ascii = Vec::<u8>::with_capacity(128);
    for i in 0..128 {
        ascii.push(i);
    }

    
    assert_eq!("AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7P\
                D0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn8\
                =",
               ascii.to_base64(BASE64_PEM));

}

#[test]
fn encode_all_bytes() {
    let mut bytes = Vec::<u8>::with_capacity(256);
    
    for i in 0..255 {
        bytes.push(i);
    }
    bytes.push(255); //bug with "overflowing" ranges?
    
    assert_eq!(
        "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7P\
         D0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn\
         +AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6\
         /wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/w==",
        bytes.to_base64(BASE64_PEM));
}
