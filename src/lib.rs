#![allow(
dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals,
unused_mut
)]

mod decode;
mod identify;
mod version_db;

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
#[derive(Copy, Clone)]
pub struct Decoder {
    pub image: *mut u8,
    pub pixels: *mut quirc_pixel_t,
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Copy, Clone)]
pub struct CapStone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [Point; 4],
    pub center: Point,
    pub c: [f64; 8],
    pub qr_grid: i32,
}
#[derive(Copy, Clone)]
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
pub type quirc_pixel_t = u8;

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

/* This enum describes the various decoder errors which may occur. */
pub type quirc_decode_error_t = u32;
pub const QUIRC_ERROR_DATA_UNDERFLOW: quirc_decode_error_t = 7;
pub const QUIRC_ERROR_DATA_OVERFLOW: quirc_decode_error_t = 6;
pub const QUIRC_ERROR_UNKNOWN_DATA_TYPE: quirc_decode_error_t = 5;
pub const QUIRC_ERROR_DATA_ECC: quirc_decode_error_t = 4;
pub const QUIRC_ERROR_FORMAT_ECC: quirc_decode_error_t = 3;
pub const QUIRC_ERROR_INVALID_VERSION: quirc_decode_error_t = 2;
pub const QUIRC_ERROR_INVALID_GRID_SIZE: quirc_decode_error_t = 1;
pub const QUIRC_SUCCESS: quirc_decode_error_t = 0;

/* Construct a new QR-code recognizer. This function will return NULL
 * if sufficient memory could not be allocated.
 */
#[no_mangle]
pub unsafe extern "C" fn quirc_new() -> *mut Decoder {
    let mut q: *mut Decoder = malloc(::std::mem::size_of::<Decoder>() as u64) as *mut Decoder;
    if q.is_null() {
        return 0 as *mut Decoder;
    } else {
        memset(
            q as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<Decoder>() as u64,
        );
        return q;
    };
}
/* Destroy a QR-code recognizer. */
#[no_mangle]
pub unsafe extern "C" fn quirc_destroy(mut q: *mut Decoder) -> () {
    free((*q).image as *mut libc::c_void);
    /* q->pixels may alias q->image when their type representation is of the
	   same size, so we need to be careful here to avoid a double free */
    if ::std::mem::size_of::<u8>() as u64
        != ::std::mem::size_of::<quirc_pixel_t>() as u64
        {
            free((*q).pixels as *mut libc::c_void);
        }
    free((*q).row_average as *mut libc::c_void);
    free(q as *mut libc::c_void);
}
/* Resize the QR-code recognizer. The size of an image must be
 * specified before codes can be analyzed.
 *
 * This function returns 0 on success, or -1 if sufficient memory could
 * not be allocated.
 */
#[no_mangle]
pub unsafe extern "C" fn quirc_resize(
    mut q: *mut Decoder,
    mut w: i32,
    mut h: i32,
) -> i32 {
    let mut olddim: usize = 0;
    let mut newdim: usize = 0;
    let mut min: usize = 0;
    let mut current_block: u64;
    let mut image: *mut u8 = 0 as *mut u8;
    let mut pixels: *mut quirc_pixel_t = 0 as *mut quirc_pixel_t;
    let mut row_average: *mut i32 = 0 as *mut i32;
    /*
     * XXX: w and h should be usize (or at least unsigned) as negatives
     * values would not make much sense. The downside is that it would break
     * both the API and ABI. Thus, at the moment, let's just do a sanity
     * check.
     */
    if !(w < 0i32 || h < 0i32) {
        /*
         * alloc a new buffer for q->image. We avoid realloc(3) because we want
         * on failure to be leave `q` in a consistant, unmodified state.
         */
        image = calloc(w as u64, h as u64) as *mut u8;
        if !image.is_null() {
            /* compute the "old" (i.e. currently allocated) and the "new"
	   (i.e. requested) image dimensions */
            olddim = ((*q).w * (*q).h) as usize;
            newdim = (w * h) as usize;
            min = if olddim < newdim { olddim } else { newdim };
            /*
             * copy the data into the new buffer, avoiding (a) to read beyond the
             * old buffer when the new size is greater and (b) to write beyond the
             * new buffer when the new size is smaller, hence the min computation.
             */
            memcpy(
                image as *mut libc::c_void,
                (*q).image as *const libc::c_void,
                min,
            );
            /* alloc a new buffer for q->pixels if needed */
            if ::std::mem::size_of::<u8>() as u64
                != ::std::mem::size_of::<quirc_pixel_t>() as u64
                {
                    pixels = calloc(
                        newdim,
                        ::std::mem::size_of::<quirc_pixel_t>() as u64,
                    ) as *mut quirc_pixel_t;
                    if pixels.is_null() {
                        current_block = 12811913577533268975;
                    } else {
                        current_block = 12675440807659640239;
                    }
                } else {
                current_block = 12675440807659640239;
            }
            match current_block {
                12811913577533268975 => {}
                _ => {
                    /* alloc a new buffer for q->row_average */
                    row_average = calloc(
                        w as u64,
                        ::std::mem::size_of::<i32>() as u64,
                    ) as *mut i32;
                    if !row_average.is_null() {
                        /* alloc succeeded, update `q` with the new size and buffers */
                        (*q).w = w;
                        (*q).h = h;
                        free((*q).image as *mut libc::c_void);
                        (*q).image = image;
                        if ::std::mem::size_of::<u8>() as u64
                            != ::std::mem::size_of::<quirc_pixel_t>() as u64
                            {
                                free((*q).pixels as *mut libc::c_void);
                                (*q).pixels = pixels
                            }
                        free((*q).row_average as *mut libc::c_void);
                        (*q).row_average = row_average;
                        return 0i32;
                    }
                }
            }
        }
    }
    free(image as *mut libc::c_void);
    free(pixels as *mut libc::c_void);
    free(row_average as *mut libc::c_void);
    return -1i32;
}
/* Return a string error message for an error code. */
#[no_mangle]
pub unsafe extern "C" fn quirc_strerror(mut err: quirc_decode_error_t) -> *const u8 {
    if err as u32 >= 0i32 as u32
        && (err as u64)
        < (::std::mem::size_of::<[*const u8; 8]>() as u64)
        .wrapping_div(::std::mem::size_of::<*const u8>() as u64)
        {
            return error_table[err as usize];
        } else {
        return b"Unknown error\x00" as *const u8 as *const u8;
    };
}
static mut error_table: [*const u8; 8] = unsafe {
    [
        b"Success\x00" as *const u8 as *const u8,
        b"Invalid grid size\x00" as *const u8 as *const u8,
        b"Invalid version\x00" as *const u8 as *const u8,
        b"Format data ECC failure\x00" as *const u8 as *const u8,
        b"ECC failure\x00" as *const u8 as *const u8,
        b"Unknown data type\x00" as *const u8 as *const u8,
        b"Data overflow\x00" as *const u8 as *const u8,
        b"Data underflow\x00" as *const u8 as *const u8,
    ]
};
/* Return the number of QR-codes identified in the last processed
 * image.
 */
#[no_mangle]
pub unsafe extern "C" fn quirc_count(mut q: *const Decoder) -> i32 {
    return (*q).num_grids;
}
