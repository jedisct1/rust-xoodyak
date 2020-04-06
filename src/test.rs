use crate::*;

#[test]
fn test_keyed_empty() {
    let mut st = XoodyakKeyed::new(b"key", None, None).unwrap();
    let mut out = [0u8; 32];
    st.squeeze(&mut out);
    assert_eq!(
        out,
        [
            66, 178, 202, 148, 57, 130, 63, 168, 102, 164, 133, 23, 92, 12, 119, 20, 101, 183, 71,
            108, 223, 209, 174, 181, 31, 244, 152, 114, 126, 97, 63, 85
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
    let mut st = XoodyakKeyed::new(b"key", None, None).unwrap();
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
            156, 171, 4, 39, 106, 5, 25, 178, 171, 147, 232, 225, 100, 52, 172, 122, 5, 151, 0,
            119, 124, 249, 88, 2, 180, 192, 83, 103, 207, 102, 36, 148
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
            155, 143, 233, 235, 29, 90, 44, 7, 195, 61, 190, 250, 118, 127, 179, 127, 224, 222,
            204, 66, 172, 164, 36, 75, 56, 117, 23, 133, 128, 122, 170, 120
        ]
    );
    st.absorb(&m[..]);
    let hash = st.squeeze_to_vec(32);
    assert_eq!(
        hash,
        [
            96, 33, 188, 162, 251, 204, 83, 135, 128, 18, 64, 123, 175, 140, 38, 97, 218, 185, 96,
            119, 131, 192, 30, 72, 155, 134, 38, 151, 216, 3, 243, 179
        ]
    );
}

#[test]
fn test_aead() {
    let mut st = XoodyakKeyed::new(b"key", None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    let ad = b"ad";
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let c = st
        .aead_encrypt_to_vec(Some(&nonce), Some(ad), Some(m))
        .unwrap();

    let mut st = st0.clone();
    let m2 = st.aead_decrypt_to_vec(Some(&nonce), Some(ad), &c).unwrap();
    assert_eq!(&m[..], &m2[..]);

    let mut st = st0.clone();
    let xm2 = st.aead_decrypt_to_vec(Some(&nonce), None, &c);
    assert!(xm2.is_err());

    let mut st = st0.clone();
    let xm2 = st.aead_decrypt_to_vec(None, Some(ad), &c);
    assert!(xm2.is_err());

    let mut st = st0.clone();
    let xm2 = st.aead_decrypt_to_vec(Some(&nonce), Some(ad), &m[..]);
    assert!(xm2.is_err());

    let mut st = XoodyakKeyed::new(b"Another key", None, None).unwrap();
    let xm2 = st.aead_decrypt_to_vec(Some(&nonce), Some(ad), &m[..]);
    assert!(xm2.is_err());
}

#[test]
fn test_aead_in_place() {
    let mut st = XoodyakKeyed::new(b"key", None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    let ad = b"ad";
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let c = st.aead_encrypt_in_place_to_vec(Some(&nonce), Some(ad), m.to_vec());

    let mut st = st0.clone();
    let m2 = st
        .aead_decrypt_in_place_to_vec(Some(&nonce), Some(ad), c)
        .unwrap();
    assert_eq!(&m[..], &m2[..]);
}

#[test]
fn test_aead_detached() {
    let mut st = XoodyakKeyed::new(b"key", None, None).unwrap();
    let st0 = st.clone();
    let m = b"message";
    let ad = b"ad";
    let nonce = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let (c, auth_tag) = st
        .aead_encrypt_to_vec_detached(Some(&nonce), Some(ad), Some(m))
        .unwrap();

    let mut st = st0.clone();
    let expected_tag = [
        218, 40, 53, 178, 223, 20, 7, 11, 169, 104, 239, 91, 55, 200, 152, 109,
    ];
    assert_eq!(auth_tag.as_ref(), expected_tag);
    let m2 = st
        .aead_decrypt_to_vec_detached(expected_tag.into(), Some(&nonce), Some(ad), Some(&c))
        .unwrap();
    assert_eq!(m2, m);
}
