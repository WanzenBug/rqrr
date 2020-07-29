use rqrr::{PreparedImage, DeQRError};
use image;

#[test]
fn test_invalid_version() {
    let img = image::open("tests/data/errors/invalid_version.gif").unwrap().to_luma();

    let mut search_img = PreparedImage::prepare(img);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let decoded = grids[0].decode();
    assert!(decoded.is_err());
    
    let err = decoded.unwrap_err();
    assert_eq!(err, DeQRError::InvalidVersion);
}

#[test]
fn test_format_ecc() {
    let img = image::open("tests/data/errors/format_ecc.png").unwrap().to_luma();

    let mut search_img = PreparedImage::prepare(img);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let decoded = grids[0].decode();
    println!("{:?}", decoded);
    assert!(decoded.is_err());
    
    let err = decoded.unwrap_err();
    assert_eq!(err, DeQRError::FormatEcc);
}
