use crate::*;

#[test]
fn test_keyed_empty() {
    let mut st = XoodyakKeyed::new(b"key", None, None, None).unwrap();
    let mut out = [0u8; 32];
    st.squeeze(&mut out);
    assert_eq!(
        out,
        [
            106, 247, 180, 176, 207, 217, 130, 200, 237, 113, 163, 185, 224, 53, 120, 137, 251,
            126, 216, 3, 87, 45, 239, 214, 41, 201, 246, 56, 83, 55, 18, 108
        ]
    );
}

#[test]
fn test_unkeyed_empty() {
    let mut st = XoodyakHash::new();
    let mut out = [0u8; 32];
    st.squeeze(&mut out);
    assert_eq!(
        out,
        [
            141, 216, 213, 137, 191, 252, 99, 169, 25, 45, 35, 27, 20, 160, 165, 255, 204, 246,
            41, 214, 87, 39, 76, 114, 39, 130, 131, 52, 124, 189, 128, 53
        ]
    );

    let mut st = XoodyakHash::new();
    let mut out = [0u8; 32];
    st.absorb(&[]);
    st.squeeze(&mut out);
    assert_eq!(
        out,
        [
            234, 21, 47, 43, 71, 188, 226, 78, 251, 102, 196, 121, 212, 173, 241, 123, 211, 36,
            216, 6, 232, 95, 247, 94, 227, 105, 238, 80, 220, 143, 139, 209
        ]
    );
}

#[test]
fn test_encrypt() {
    let mut st = XoodyakKeyed::new(b"key", None, None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    let mut c = [0u8; 7];
    st.encrypt(&mut c, m).unwrap();

    let mut st = st0.clone();
    let mut m2 = [0u8; 7];
    st.decrypt(&mut m2, &c).unwrap();
    assert_eq!(&m[..], &m2[..]);

    let mut st = st0.clone();
    st.ratchet();
    let mut m2 = [0u8; 7];
    st.decrypt(&mut m2, &c).unwrap();
    assert_ne!(&m[..], &m2[..]);

    let c0 = c;
    let mut st = st0.clone();
    st.decrypt_in_place(&mut c);
    assert_eq!(&m[..], &c[..]);

    let mut st = st0;
    st.encrypt_in_place(&mut c);
    assert_eq!(c0, c);

    let mut tag = [0u8; 32];
    st.squeeze(&mut tag);
    assert_eq!(
        tag,
        [
            10, 175, 140, 82, 142, 109, 23, 111, 201, 232, 32, 52, 122, 46, 254, 206, 236, 54,
            97, 165, 40, 85, 166, 91, 124, 88, 26, 144, 100, 250, 243, 157
        ]
    );
}

#[test]
fn test_unkeyed_hash() {
    let mut st = XoodyakHash::new();
    let m = b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.";
    st.absorb(&m[..]);
    let mut hash = [0u8; 32];
    st.squeeze(&mut hash);
    assert_eq!(
        hash,
        [
            144, 82, 141, 27, 59, 215, 34, 104, 197, 106, 251, 142, 112, 235, 111, 168, 19, 6,
            112, 222, 160, 168, 230, 38, 27, 229, 248, 179, 94, 227, 247, 25
        ]
    );
    st.absorb(&m[..]);
    let mut hash = [0u8; 32];
    st.squeeze(&mut hash);
    assert_eq!(
        hash,
        [
            102, 50, 250, 132, 79, 91, 248, 161, 121, 248, 225, 33, 105, 159, 111, 230, 135, 252,
            43, 228, 152, 41, 58, 242, 211, 252, 29, 234, 181, 0, 196, 220
        ]
    );
}

#[test]
fn test_aead() {
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut st = XoodyakKeyed::new(b"key", Some(&nonce), None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    let ad = b"ad";
    st.absorb(ad);
    let mut c = [0u8; 7 + XOODYAK_AUTH_TAG_BYTES];
    st.aead_encrypt(&mut c, Some(m)).unwrap();

    let mut st = st0.clone();
    st.absorb(ad);
    let mut m2 = [0u8; 7];
    st.aead_decrypt(&mut m2, &c).unwrap();
    assert_eq!(&m[..], &m2[..]);

    let mut st = st0;
    let mut m2 = [0u8; 7];
    let result = st.aead_decrypt(&mut m2, &m[..]);
    assert!(result.is_err());

    let mut st = XoodyakKeyed::new(b"Another key", Some(&nonce), None, None).unwrap();
    let mut m2 = [0u8; 7];
    let result = st.aead_decrypt(&mut m2, &m[..]);
    assert!(result.is_err());
}

#[test]
fn test_aead_in_place() {
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut st = XoodyakKeyed::new(b"key", Some(&nonce), None, None).unwrap();
    let st0 = st.clone();

    let m = b"message";
    st.absorb(b"ad");
    let mut buf = [0u8; 7 + XOODYAK_AUTH_TAG_BYTES];
    buf[..7].copy_from_slice(m);
    st.aead_encrypt_in_place(&mut buf).unwrap();

    let mut st = st0.clone();
    let mut buf2 = buf;
    let result = st.aead_decrypt_in_place(&mut buf2);
    assert!(result.is_err());

    let mut st = st0;
    st.absorb(b"ad");
    let m2 = st.aead_decrypt_in_place(&mut buf).unwrap();
    assert_eq!(&m[..], &m2[..]);
}

#[test]
fn test_aead_detached() {
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut st = XoodyakKeyed::new(b"key", Some(&nonce), None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    st.absorb(b"ad");
    let mut c = [0u8; 7];
    let auth_tag = st.aead_encrypt_detached(&mut c, Some(m)).unwrap();

    let mut st = st0;
    let expected_tag = [
        12, 91, 0, 120, 191, 214, 119, 66, 122, 225, 184, 239, 213, 214, 247, 57,
    ];
    assert_eq!(auth_tag.as_ref(), expected_tag);
    st.absorb(b"ad");
    let mut m2 = [0u8; 7];
    st.aead_decrypt_detached(&mut m2, &expected_tag.into(), Some(&c))
        .unwrap();
    assert_eq!(&m2[..], &m[..]);
}
