//! Find and read QR-Codes
//!
//! This crates exports functions and types that can be used to search for QR-Codes in images and
//! decode them.
//!
//!
//! # Usage
//! The most basic usage is shown below:
//!
//! ```
//! use image;
//! use rqrr;
//!
//! let img = image::open("QR-Code.jpg").unwrap();
//! let codes = rqrr::find_and_decode_from_image(&img);
//! assert_eq!(codes.len(), 1);
//! assert_eq!(codes[0].val, "http://qr2.it/Go/24356");
//! ```
//!
//! If you have some other form of picture storage, you can use [`find_and_decode_from_func`]().
//! This allows you define your own source for images.
//! ```
//! use image;
//! use rqrr;
//!
//! let img = image::open("QR-Code.jpg").unwrap().to_luma();
//! let w = img.width() as usize;
//! let h = img.height() as usize;
//! let codes = rqrr::find_and_decode(w, h, |x, y|  img.get_pixel(x as u32, y as u32).data[0]);
//! assert_eq!(codes.len(), 1);
//! assert_eq!(codes[0].val, "http://qr2.it/Go/24356");
//! ```
#[cfg(feature = "debug-plot")]
use gnuplot;
#[cfg(feature = "img")]
use image;

pub use self::decode::{MetaData, Version, decode};
use self::identify::{CapStone, capstones_from_image, find_groupings, Grid, Image, Point};

pub mod decode;
pub mod identify;
pub mod version_db;


#[derive(Debug)]
pub struct Poly([Point; 4]);

pub trait GridImage {
    fn size(&self) -> usize;
    fn bit(&self, x: usize, y: usize) -> bool;
}

#[derive(Debug, Clone)]
pub struct SimpleGridImage {
    cell_bitmap: Vec<u8>,
    size: usize,
}

impl SimpleGridImage {
    pub fn from_func<F>(size: usize, fill_func: F) -> Self where F: Fn(usize, usize) -> bool {
        let mut cell_bitmap = vec![0; (size * size + 7) / 8];
        let mut c = 0;
        for y in 0..size {
            for x in 0..size {
                if fill_func(x, y) {
                    cell_bitmap[c >> 3] |= 1 << (c & 7) as u8;
                }
                c += 1;
            }
        }

        SimpleGridImage {
            cell_bitmap,
            size,
        }
    }
}


#[derive(Debug)]
pub struct Code {
    pub meta: MetaData,
    pub val: String,
    pub position: Poly,
}

#[cfg(feature = "img")]
pub fn find_and_decode_from_image(img: &image::DynamicImage) -> Vec<Code> {
    let img = img.to_luma();
    let w = img.width() as usize;
    let h = img.height() as usize;

    find_and_decode_from_func(w, h, |x, y| {
        img.get_pixel(x as u32, y as u32).data[0]
    })
}


/// Find QR-Codes and decode them
///
/// This method expects to be given an image of dimension `width` * `height`. The data is supplied
/// via the `fill` function. The fill function will be called width coordinates `x, y`, where `x`
/// ranges from 0 to `width` and `y` from 0 to `height`. The return value is expected to correspond
/// with the greyscale value of the image to decode, 0 being black, 255 being white.
///
/// # Returns
///
/// Returns a collection of all QR-Codes that have been found. They are already decoded and contain
/// some extra metadata like position in the image as well as information about the used QR code
/// itself
///
/// # Panics
///
/// Panics if `width * height` would overflow.
///
/// # Examples
///
/// ```
/// use image;
/// use rqrr;
///
/// let img = image::open("QR-Code.jpg").unwrap().to_luma();
/// let w = img.width() as usize;
/// let h = img.height() as usize;
/// let codes = rqrr::find_and_decode(w, h, |x, y|  img.get_pixel(x as u32, y as u32).data[0]);
/// assert_eq!(codes.len(), 1);
/// assert_eq!(codes[0].val, "http://qr2.it/Go/24356");
/// ```
pub fn find_and_decode_from_func<F>(width: usize, height: usize, fill: F) -> Vec<Code> where F: FnMut(usize, usize) -> u8 {
    let mut img = Image::from_greyscale(width, height, fill);
    let mut ret = Vec::new();
    let caps = capstones_from_image(&mut img);
    let groups = find_groupings(caps);
    let grids: Vec<_> = groups.into_iter()
        .filter_map(|group| Grid::from_group(&mut img, group))
        .collect();

    for grid in grids {
        let mut decode_val = Vec::new();

        let grid_img = grid.into_grid_image(&img);

        let meta = match decode::decode(&grid_img, &mut decode_val) {
            Ok(x) => x,
            Err(_) => continue,
        };

        let position = Poly([Default::default(); 4]);
        let val = match String::from_utf8(decode_val) {
            Ok(x) => x,
            Err(_) => continue,
        };

        ret.push(Code {
            meta,
            val,
            position,
        })
    }

    ret
}

impl GridImage for SimpleGridImage {
    fn size(&self) -> usize {
        self.size
    }

    fn bit(&self, x: usize, y: usize) -> bool {
        let c = y * self.size + x;
        self.cell_bitmap[c >> 3] & (1 << (c & 7) as u8) != 0
    }
}


#[derive(Debug)]
pub enum DeQRError {
    IoError,
    DataUnderflow,
    DataOverflow,
    UnknownDataType,
    DataEcc,
    FormatEcc,
    InvalidVersion,
    InvalidGridSize,
}

pub type DeQRResult<T> = Result<T, DeQRError>;
