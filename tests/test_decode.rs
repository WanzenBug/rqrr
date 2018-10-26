extern crate qrcodegen;
extern crate png;
extern crate de_qr;

use std::io::Write;

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
            max  = std::cmp::max(i >> 3, max);
            if code.get_module(x as i32, y as i32) {
                buf[i >> 3] |= 1 << ((i & 7) as u8);
                v.push(0);
            } else {
                v.push(255);
            }

        }
    }
    match code.error_correction_level() {
        qrcodegen::QrCodeEcc::Low => eprintln!("code.error_correction_level() = Low"),
        qrcodegen::QrCodeEcc::Medium => eprintln!("code.error_correction_level() = Medium"),
        qrcodegen::QrCodeEcc::High => eprintln!("code.error_correction_level() = High"),
        qrcodegen::QrCodeEcc::Quartile => eprintln!("code.error_correction_level() = Quartile"),
    }
    let s= code.to_svg_string(100);
    let mut f = std::fs::File::create("qr.svg").unwrap();
    f.write_all(s.as_bytes()).unwrap();
    drop(f);

    {
        use png::HasParameters;
        let f = std::fs::File::create("qr.png").unwrap();
        let mut enc = png::Encoder::new(f, code.size() as u32, code.size() as u32);
        enc.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(&v[..]).unwrap()
    }

    eprintln!("code.mask() = {:#?}", code.mask().value());

    eprintln!("code.size() = {:#?}", code.size());
    for b in buf.iter() {
        eprint!("\\x{:02X}", b)
    }
    eprintln!();
    let code = de_qr::Code {
        size: code.size() as usize,
        cell_bitmap: buf,
        corners: Default::default(),
    };
    let dec = code.decode().unwrap();
    eprintln!("dec.payload[..dec.payload_len] = {:#?}", &dec.payload[..dec.payload_len]);
}
