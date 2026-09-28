pub use self::grid::SkewedGridLocation;
pub use self::match_capstones::{
    find_and_rank_possible_neighbors, RELAXED_VIABILITY_THRESHOLD, VIABILITY_THRESHOLD,
};

pub mod grid;
pub mod match_capstones;

/// A simple point in (some) space
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
