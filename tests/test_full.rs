#![cfg(feature = "img")]

use std::collections::HashSet;

#[test]
fn test_full_time() {
    let jpg = image::open("tests/data/full/gogh.jpg").unwrap().to_luma8();

    let mut search_img = rqrr::PreparedImage::prepare(jpg);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 3);

    let mut codes = HashSet::new();
    for grid in grids {
        let (_meta, content) = grid.decode().unwrap();
        codes.insert(content);
    }
    let mut ref_set = HashSet::new();
    ref_set.insert("https://github.com/WanzenBug/rqrr".to_string());
    ref_set.insert("rqrr".to_string());
    ref_set.insert("1234567891011121314151617181920".to_string());
    assert_eq!(codes, ref_set);
}

#[test]
fn test_full_large_version() {
    let gif = image::open("tests/data/full/superlong.gif")
        .unwrap()
        .to_luma8();

    let mut search_img = rqrr::PreparedImage::prepare(gif);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let (meta, content) = grids[0].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(14));
    assert_eq!(content, "superlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdatasuperlongdata");
}

#[test]
fn test_full_multi() {
    let png = image::open("tests/data/full/multiple.png")
        .unwrap()
        .to_luma8();

    let mut search_img = rqrr::PreparedImage::prepare(png);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 3);

    let (meta, content) = grids[0].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(13));
    assert_eq!(content, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariat");

    let (meta, content) = grids[1].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(13));
    assert_eq!(content, "ur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea comm");

    let (meta, content) = grids[2].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(11));
    assert_eq!(content, "odo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");
}

#[test]
fn test_full_multi_rotated() {
    let png = image::open("tests/data/full/multiple_rotated.png")
        .unwrap()
        .to_luma8();

    let mut search_img = rqrr::PreparedImage::prepare(png);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 3);

    let (meta, content) = grids[0].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(13));
    assert_eq!(content, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariat");

    let (meta, content) = grids[1].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(13));
    assert_eq!(content, "ur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea comm");

    let (meta, content) = grids[2].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(11));
    assert_eq!(content, "odo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");
}

#[test]
fn test_mirrored() {
    let img = image::open("tests/data/mirrored.gif").unwrap().to_luma8();

    let mut search_img = rqrr::PreparedImage::prepare(img);
    let grids = search_img.detect_grids();
    assert_eq!(grids.len(), 1);

    let (meta, raw) = grids[0].decode().unwrap();
    assert_eq!(meta.version, rqrr::Version(1));
    assert_eq!(meta.ecc_level, 0);
    assert_eq!(meta.mask, 0);
    assert_eq!(raw, "rqrr");
}
