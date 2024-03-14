# rust-qr-reader - Find and read QR-Codes
[![documentation](https://docs.rs/rqrr/badge.svg)](https://docs.rs/rqrr/)
[![Build Status](https://travis-ci.com/WanzenBug/rqrr.svg?branch=master)](https://travis-ci.com/WanzenBug/rqrr)
[![Build Status](https://github.com/WanzenBug/rqrr/actions/workflows/CI.yaml/badge.svg?branch=master)](https://github.com/WanzenBug/rqrr/actions/workflows/CI.yaml)

This crates exports functions and types that can be used to search for QR-Codes in images and
decode them.

## Usage
The most basic usage is shown below:

```rust
use image;
use rqrr;

let img = image::open("tests/data/github.gif")?.to_luma();
// Prepare for detection
let mut img = rqrr::PreparedImage::prepare(img);
// Search for grids, without decoding
let grids = img.detect_grids();
assert_eq!(grids.len(), 1);
// Decode the grid
let (meta, content) = grids[0].decode()?;
assert_eq!(meta.ecc_level, 0);
assert_eq!(content, "https://github.com/WanzenBug/rqrr");
```
For more information visit [docs.rs](https://docs.rs/rqrr/)

## License
Either [APACHE](LICENSE-APACHE) or [MIT](LICENSE-MIT)

## Attribution
This library was made on the base of [quirc](https://github.com/dlbeer/quirc)
