#![allow(
dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals,
unused_mut
)]

mod decode;
mod galois;
// mod identify;
mod version_db;

pub use decode::{Code, Data};
/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
#[derive(Clone)]
pub struct Decoder {
    pub image: *mut u8,
    pub pixels: *mut u8,
    pub row_average: *mut i32,
    pub w: i32,
    pub h: i32,
    pub num_regions: i32,
    pub regions: [Region; 254],
    pub num_capstones: i32,
    pub capstones: [CapStone; 32],
    pub num_grids: i32,
    pub grids: [Grid; 8],
}
#[derive(Debug, Clone)]
pub struct Grid {
    pub caps: [i32; 3],
    pub align_region: i32,
    pub align: Point,
    pub tpep: [Point; 3],
    pub hscan: i32,
    pub vscan: i32,
    pub grid_size: i32,
    pub c: [f64; 8],
}
/* This structure describes a location in the input image buffer. */
#[derive(Debug, Clone, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone)]
pub struct CapStone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [Point; 4],
    pub center: Point,
    pub c: [f64; 8],
    pub qr_grid: i32,
}
#[derive(Debug, Clone)]
pub struct Region {
    pub seed: Point,
    pub count: i32,
    pub capstone: i32,
}
/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
#[derive(Debug)]
pub enum DeQRError {
    DATA_UNDERFLOW,
    DATA_OVERFLOW,
    UNKNOWN_DATA_TYPE,
    DATA_ECC,
    FORMAT_ECC,
    INVALID_VERSION,
    INVALID_GRID_SIZE,
}

pub type DeQRResult<T> = Result<T, DeQRError>;
