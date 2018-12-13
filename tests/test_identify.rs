use rqrr;
use image;

fn test_data(buffer: &[u8]) -> Vec<rqrr::Code> {
    let img = image::load_from_memory(buffer).unwrap();
    rqrr::find_and_decode_from_image(&img)
}

#[test]
fn test_identify_code1() {
    let buffer = include_bytes!("data/code1.jpg");
    let codes= test_data(buffer);

    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].val, "bla");
}


#[test]
fn test_identify_code2() {
    let buffer = include_bytes!("data/code2.jpg");
    let codes= test_data(buffer);

    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].val, "bla");
}


#[test]
fn test_identify_nocode1() {
    let buffer = include_bytes!("data/nocode1.jpg");
    let codes= test_data(buffer);

    assert_eq!(codes.len(), 0);
}
