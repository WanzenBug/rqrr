# rust-qr-reader - Find and read QR-Codes
[![documentation](https://docs.rs/rqrr/badge.svg)](https://docs.rs/rqrr/)
[![Build Status](https://travis-ci.com/WanzenBug/rqrr.svg?branch=master)](https://travis-ci.com/WanzenBug/rqrr)

This crates exports functions and types that can be used to search for QR-Codes in images and
decode them.

## Usage
The most basic usage is shown below:

```rust
use image;
use rqrr;

let img = image::open("tests/data/github.gif").unwrap();
let codes = rqrr::find_and_decode_from_image(&img);
assert_eq!(codes.len(), 1);
assert_eq!(codes[0].val, "https://github.com/WanzenBug/rqrr");
```
For more information visit [docs.rs](https://docs.rs/rqrr/)

## License
Either [APACHE](LICENSE-APACHE) or [MIT](LICENSE-MIT)

## Attribution
This library was made on the base of [quirc](https://github.com/dlbeer/quirc)
