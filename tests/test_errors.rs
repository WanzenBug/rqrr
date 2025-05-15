#![cfg(feature = "img")]

use rqrr::{DeQRError, PreparedImage};

use std::io::{Error, ErrorKind, Write};

struct BrokenWriter {}

impl Write for BrokenWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        println!(
            "Writer got this data: {:?} (decoded: {:?})",
            buf,
            String::from_utf8_lossy(buf)
        );
        Err(Error::new(ErrorKind::PermissionDenied, "testing IoError"))
    }
    fn flush(&mut self) -> Result<(), Error> {
        println!("Writer was flushed");
        Err(Error::new(ErrorKind::PermissionDenied, "testing IoError"))
    }
}

#[test]
fn test_io_error() {
    let img = image::open("tests/data/errors/io_error.png")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    let grids = search_img.detect_grids();

    assert_eq!(grids.len(), 1);

    let writer = BrokenWriter {};

    let result = grids[0].decode_to(writer);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, DeQRError::IoError);
}

#[test]
fn test_invalid_version() {
    let img = image::open("tests/data/errors/invalid_version.gif")
        .unwrap()
        .to_luma8();

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
    let img = image::open("tests/data/errors/format_ecc.png")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let decoded = grids[0].decode();
    println!("{:?}", decoded);
    assert!(decoded.is_err());

    let err = decoded.unwrap_err();
    assert_eq!(err, DeQRError::FormatEcc);
}

#[test]
fn test_data_ecc() {
    let img = image::open("tests/data/errors/data_ecc.png")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let decoded = grids[0].decode();
    println!("{:?}", decoded);
    assert!(decoded.is_err());

    let err = decoded.unwrap_err();
    assert_eq!(err, DeQRError::DataEcc);
}

#[test]
fn test_should_not_panic() {
    let img = image::open("tests/data/errors/should-not-panic-1.jpg")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    let _ = search_img.detect_grids();

    let img = image::open("tests/data/errors/should-not-panic-2.jpg")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    let _ = search_img.detect_grids();
}

#[test]
fn test_should_detect_grid() {
    let img = image::open("tests/data/errors/should_detect_grid.png")
        .unwrap()
        .to_luma8();

    let mut search_img = PreparedImage::prepare(img);
    assert!(search_img.detect_grids().len() > 0);
}
// As of commit 956686877c964731559463dc645aa14e44e691b3, the following elements
// of the DeQRError enum are not used anywhere:
// - InvalidGridSize
// - EncodingError

// Also, these errors require grids to be manipulated at a deep level, which
// requires involved knowledge of the QR code structure:
// - DataUnderflow
// - DataOverflow
// - UnknownDataType
