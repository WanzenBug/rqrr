//! Find and read QR-Codes
//!
//! This crates exports functions and types that can be used to search for QR-Codes in images and
//! decode them.
//!
//!
//! # Usage
//! The most basic usage is shown below:
//!
//! ```rust
//! use image;
//! use rqrr;
//!
//! let img = image::open("tests/data/github.gif").unwrap();
//! let codes = rqrr::find_and_decode_from_image(&img);
//! assert_eq!(codes.len(), 1);
//! assert_eq!(codes[0].val, "https://github.com/WanzenBug/rqrr");
//! ```
//!
//! If you have some other form of picture storage, you can use [`find_and_decode_from_func`]().
//! This allows you define your own source for images.

#[cfg(feature = "img")]
use image;

pub use self::decode::{decode, MetaData, Version};
pub use self::identify::{CapStone, capstones_from_image, find_groupings, Point, SearchableImage, SkewedGridLocation, SearchableImageBuffer, BasicImageBuffer};

mod decode;
mod identify;
mod version_db;

/// A grid that contains exactly one QR code square.
///
/// The common trait for everything that can be decoded as a QR code. Given a normal image, we first
/// need to find the QR grids in it. See [capstones_from_image](fn.find_capstones_from_image.html),
/// [find_groupings](fn.find_groupings.html) and
/// [SkewedGridLocation](struct.SkewedGridLocation.html).
///
/// This trait can be implemented when some object is known to be exactly the bit-pattern
/// of a QR code.
pub trait Grid {
    /// Return the size of the grid.
    ///
    /// Since QR codes are always squares, the grid is assumed to be size * size.
    fn size(&self) -> usize;

    /// Return the value of the bit at the given location.
    ///
    /// `true` means 'black', `false` means 'white'
    fn bit(&self, y: usize, x: usize) -> bool;

    #[cfg(feature = "img")]
    fn write_grid_to(&self, p: &str) {
        let mut dyn_img = image::GrayImage::new(self.size() as u32, self.size() as u32);
        for y in 0..self.size() {
            for x in 0..self.size() {
                let color = match self.bit(x, y) {
                    true => 0,
                    false => 255,
                };
                dyn_img.get_pixel_mut(x as u32, y as u32).data[0] = color;
            }
        }
        dyn_img.save(p).unwrap();
    }
}

/// A basic GridImage that can be generated from a given function.
///
/// # Example
///
/// ```rust
/// use rqrr;
///
/// let grid = [
///     [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, ],
///     [1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, ],
///     [1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, ],
///     [1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, ],
///     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ],
///     [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, ],
///     [1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, ],
///     [0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, ],
///     [1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, ],
///     [0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, ],
///     [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, ],
///     [1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, ],
///     [1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, ],
///     [1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, ],
///     [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, ],
///     [1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, ],
/// ];
///
/// let img = rqrr::SimpleGrid::from_func(21, |x, y| {
///     grid[y][x] == 1
/// });
/// let mut result = Vec::new();
/// rqrr::decode(&img, &mut result).unwrap();
/// assert_eq!(b"rqrr".as_ref(), &result[..])
/// ```
#[derive(Debug, Clone)]
pub struct SimpleGrid {
    cell_bitmap: Vec<u8>,
    size: usize,
}

impl SimpleGrid {
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

        SimpleGrid {
            cell_bitmap,
            size,
        }
    }
}


impl Grid for SimpleGrid {
    fn size(&self) -> usize {
        self.size
    }

    fn bit(&self, y: usize, x: usize) -> bool {
        let c = y * self.size + x;
        self.cell_bitmap[c >> 3] & (1 << (c & 7) as u8) != 0
    }
}


/// The decoded content of a QR-Code
///
/// The member `val` stores the decoded value of a QR-Code.
/// The member `meta` stores [MetaData](struct.MetaData.html) (Version number, ECC-Level, etc)
/// The member `position` stores the 4 'corners' of the QR code, in image coordinates.
#[derive(Debug)]
pub struct Code {
    pub meta: MetaData,
    pub val: String,
    pub position: [Point; 4],
}


/// Given a image object, locate all codes found in it
///
/// This is a convenient wrapper if you use the `image` crate already. The only requirement
/// for the image is that the 'black' parts of a QR code are 'dark', 'white' parts 'bright'.
///
/// # Example
/// ```rust
/// use image;
/// use rqrr;
///
/// let img = image::open("tests/data/github.gif").unwrap();
/// let codes = rqrr::find_and_decode_from_image(&img);
/// assert_eq!(codes.len(), 1);
/// assert_eq!(codes[0].val, "https://github.com/WanzenBug/rqrr");
/// ```
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
/// let img = image::open("tests/data/github.gif").unwrap().to_luma();
/// let w = img.width() as usize;
/// let h = img.height() as usize;
/// let codes = rqrr::find_and_decode_from_func(w, h, |x, y|  img.get_pixel(x as u32, y as u32).data[0]);
/// assert_eq!(codes.len(), 1);
/// assert_eq!(codes[0].val, "https://github.com/WanzenBug/rqrr");
/// ```
pub fn find_and_decode_from_func<F>(width: usize, height: usize, fill: F) -> Vec<Code> where F: FnMut(usize, usize) -> u8 {
    let mut img = SearchableImage::from_greyscale(width, height, fill);
    let caps = capstones_from_image(&mut img);
    let groups = find_groupings(caps);
    let grids: Vec<_> = groups.into_iter()
        .filter_map(|group| SkewedGridLocation::from_group(&mut img, group))
        .collect();
    let mut ret = Vec::new();
    for grid in grids {
        let mut decode_val = Vec::new();
        let position = [
            grid.c.map(0.0, 0.0),
            grid.c.map(grid.grid_size as f64 + 1.0, 0.0),
            grid.c.map(grid.grid_size as f64 + 1.0, grid.grid_size as f64 + 1.0),
            grid.c.map(0.0, grid.grid_size as f64 + 1.0),
        ];

        let grid_img = grid.into_grid_image(&img);
        let meta = match decode::decode(&grid_img, &mut decode_val) {
            Ok(x) => x,
            Err(_) => continue,
        };

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

/// Possible errors that can happen during decoding
#[derive(Debug)]
pub enum DeQRError {
    /// Could not write the output to the output stream/string
    IoError,
    /// Expected more bits to decode
    DataUnderflow,
    /// Expected less bits to decode
    DataOverflow,
    /// Unknown data type in encoding
    UnknownDataType,
    /// Could not correct errors / code corrupt
    DataEcc,
    /// Could not read format information from both locations
    FormatEcc,
    /// Unsupported / non-existent version read
    InvalidVersion,
    /// Unsupported / non-existent grid size read
    InvalidGridSize,
}

pub type DeQRResult<T> = Result<T, DeQRError>;
