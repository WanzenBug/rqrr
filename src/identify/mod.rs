pub use self::capstone::{CapStone, capstones_from_image};
pub use self::grid::SkewedGridLocation;
pub use self::helper::Perspective;
pub use self::image::{PixelColor, SearchableImage};
pub use self::match_capstones::find_groupings;

mod helper;
mod image;
mod capstone;
mod match_capstones;
mod grid;


/// A simple point in (some) space
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}


