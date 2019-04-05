mod helper;
mod image;
mod capstone;
mod match_capstones;
mod grid;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}


pub use self::image::{PixelColor, Image};
pub use self::capstone::{CapStone, capstones_from_image};
pub use self::match_capstones::{find_groupings};
pub use self::grid::Grid;
pub use self::helper::Perspective;
