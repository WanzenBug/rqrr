extern crate rqrr;
extern crate qrcodegen;

#[test]
fn test_decode() {
    let mut buf = [0; 3917];
    let mut v = Vec::new();

    let code = qrcodegen::QrCode::encode_text("abcdefghijklmnopqrstuvwxyz0123456789",
                                              qrcodegen::QrCodeEcc::High).unwrap();

    let mut max = 0;

    for y in 0..code.size() as usize {
        for x in 0..code.size() as usize {
            let i = y * code.size() as usize + x;
            max = std::cmp::max(i >> 3, max);
            if code.get_module(x as i32, y as i32) {
                buf[i >> 3] |= 1 << ((i & 7) as u8);
                v.push(0);
            } else {
                v.push(255);
            }
        }
    }

    let grid = rqrr::SimpleGridImage::from_func(code.size() as usize, |x, y| {
        code.get_module(x as i32, y as i32)
    });
    let mut vec = Vec::new();
    let dec = rqrr::decode(&grid, &mut vec).unwrap();
    assert_eq!(dec.mask, 3);
    assert_eq!(dec.ecc_level, 2);
    assert_eq!(dec.version.to_size(), code.size() as usize);
    assert_eq!(&vec[..], b"abcdefghijklmnopqrstuvwxyz0123456789".as_ref());
}
