use crate::*;

#[test]
fn test_keyed_empty() {
    let mut st = XoodyakKeyed::new(b"key", None, None, None).unwrap();
    let mut out = [0u8; 32];
    st.squeeze(&mut out);
    assert_eq!(
        out,
        [
            213, 184, 37, 35, 77, 249, 200, 247, 209, 9, 197, 95, 44, 42, 167, 99, 73, 240, 105,
            88, 84, 196, 12, 249, 32, 158, 51, 111, 180, 218, 172, 199
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
            141, 216, 213, 137, 191, 252, 99, 169, 25, 45, 35, 27, 20, 160, 165, 255, 204, 246, 41,
            214, 87, 39, 76, 114, 39, 130, 131, 52, 124, 189, 128, 53
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
    let mut c = st.encrypt_to_vec(m).unwrap();

    let mut st = st0.clone();
    let m2 = st.decrypt_to_vec(&c).unwrap();
    assert_eq!(&m[..], m2.as_slice());

    let mut st = st0.clone();
    st.ratchet();
    let m2 = st.decrypt_to_vec(&c).unwrap();
    assert_ne!(&m[..], m2.as_slice());

    let c0 = c.clone();
    let mut st = st0.clone();
    st.decrypt_in_place(&mut c);
    assert_eq!(&m[..], &c[..]);

    let mut st = st0.clone();
    st.encrypt_in_place(&mut c);
    assert_eq!(c0, c);

    let tag = st.squeeze_to_vec(32);
    assert_eq!(
        tag,
        [
            200, 147, 13, 147, 44, 209, 36, 198, 141, 254, 108, 165, 116, 184, 23, 169, 57, 92, 70,
            159, 145, 159, 191, 25, 170, 208, 227, 10, 180, 216, 162, 196
        ]
    );
}

#[test]
fn test_unkeyed_hash() {
    let mut st = XoodyakHash::new();
    let m = b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.";
    st.absorb(&m[..]);
    let hash = st.squeeze_to_vec(32);
    assert_eq!(
        hash,
        [
            144, 82, 141, 27, 59, 215, 34, 104, 197, 106, 251, 142, 112, 235, 111, 168, 19, 6, 112,
            222, 160, 168, 230, 38, 27, 229, 248, 179, 94, 227, 247, 25
        ]
    );
    st.absorb(&m[..]);
    let hash = st.squeeze_to_vec(32);
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
    let c = st.aead_encrypt_to_vec(Some(m)).unwrap();

    let mut st = st0.clone();
    st.absorb(ad);
    let m2 = st.aead_decrypt_to_vec(&c).unwrap();
    assert_eq!(&m[..], &m2[..]);

    let mut st = st0.clone();
    let xm2 = st.aead_decrypt_to_vec(&m[..]);
    assert!(xm2.is_err());

    let mut st = XoodyakKeyed::new(b"Another key", Some(&nonce), None, None).unwrap();
    let xm2 = st.aead_decrypt_to_vec(&m[..]);
    assert!(xm2.is_err());
}

#[test]
fn test_aead_in_place() {
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut st = XoodyakKeyed::new(b"key", Some(&nonce), None, None).unwrap();
    let st0 = st.clone();

    let m = b"message";
    st.absorb(b"ad");
    let c = st.aead_encrypt_in_place_to_vec(m.to_vec());

    let mut st = st0.clone();
    let xm2 = st.aead_decrypt_in_place_to_vec(c.clone());
    assert!(xm2.is_err());

    let mut st = st0.clone();
    st.absorb(b"ad");
    let m2 = st.aead_decrypt_in_place_to_vec(c).unwrap();
    assert_eq!(&m[..], &m2[..]);
}

#[test]
fn test_aead_detached() {
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut st = XoodyakKeyed::new(b"key", Some(&nonce), None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    st.absorb(b"ad");
    let (c, auth_tag) = st.aead_encrypt_to_vec_detached(Some(m)).unwrap();

    let mut st = st0.clone();
    let expected_tag = [
        107, 47, 237, 76, 96, 196, 170, 149, 234, 87, 11, 13, 167, 51, 39, 5,
    ];
    assert_eq!(auth_tag.as_ref(), expected_tag);
    st.absorb(b"ad");
    let m2 = st
        .aead_decrypt_to_vec_detached(expected_tag.into(), Some(&c))
        .unwrap();
    assert_eq!(m2, m);
}
