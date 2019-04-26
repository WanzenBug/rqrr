use rqrr;
use image;

fn load_and_find(img: &[u8]) -> Vec<rqrr::CapStone> {
    let img = image::load_from_memory(img).unwrap().to_luma();
    let w = img.width() as usize;
    let h = img.height() as usize;
    let mut img = rqrr::identify::Image::from_greyscale(w, h, |x, y| {
        img.get_pixel(x as u32, y as u32).data[0]
    });
    rqrr::capstones_from_image(&mut img)
}


#[test]
fn test_cap() {
    let caps = load_and_find(include_bytes!("data/cap/cap.png"));
    assert_eq!(1, caps.len());
}

#[test]
fn test_cap_connected() {
    let caps = load_and_find(include_bytes!("data/cap/cap_connect.png"));
    assert_eq!(0, caps.len());
}

#[test]
fn test_cap_disconnected() {
    let caps = load_and_find(include_bytes!("data/cap/cap_disconnect.png"));
    assert_eq!(0, caps.len());
}

#[test]
fn test_cap_size() {
    let caps = load_and_find(include_bytes!("data/cap/cap_size.png"));
    assert_eq!(0, caps.len());
}
