use crate::{
    identify::Point,
    geometry::Perspective,
    prepare::{ColoredRegion, PreparedImage, Row},
};
use crate::prepare::{AreaFiller, ImageBuffer, PixelColor};


/// A locator pattern of a QR grid
///
/// One of 3 corner patterns of a QR code. Can be found using a distinctive 1:1:3:1:1 pattern
/// of black-white zones.
///
/// Stores information about the corners of the capstone (NOT the grid), the center point and
/// the local `perspective` i.e. in which direction the grid is likely skewed.
#[derive(Debug, Clone)]
pub struct CapStone {
    /// The 4 corners of the capstone
    pub corners: [Point; 4],
    /// The center point of the capstone
    pub center: Point,
    /// The local perspective of the capstone, i.e. in which direction(s) the capstone is skewed.
    pub c: Perspective,
}

#[derive(Debug, Clone)]
pub struct PolygonScoreData {
    pub ref_0: Point,
    pub scores: [i32; 4],
    pub corners: [Point; 4],
}

/// Find all 'capstones' in a given image.
///
/// A Capstones is the locator pattern of a QR code. Every QR code has 3 of these in 3 corners.
/// This function finds these patterns by scanning the image line by line for a distinctive
/// 1:1:3:1:1 pattern of black-white-black-white-black zones.
///
/// Returns a vector of [CapStones](struct.CapStone.html)
pub fn capstones_from_image<S>(img: &mut PreparedImage<S>) -> Vec<CapStone> where S: ImageBuffer {
    let mut res = Vec::new();

    for y in 0..img.height() {
        let mut finder = LineScanner::new(img.get_pixel_at(0, y));
        for x in 1..img.width() {
            if let Some(linepos) = finder.advance(img.get_pixel_at(x, y)) {
                if is_capstone(img, &linepos, y) {
                    let cap = create_capstone(img, &linepos, y);
                    res.push(cap);
                }
            }
        }
    }
    res
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct LinePosition {
    left: usize,
    stone: usize,
    right: usize,
}

/// Find a possible capstone based on black/white transitions
///
/// ```bash
///     #######
///     #     #
/// --> # ### # <--
///     # ### #
///     # ### #
///     #     #
///     #######
/// ```
/// A capstone has a distinctive pattern of 1:1:3:1:1 of black-white transitions.
/// So a run of black is followed by a run of white of equal length, followed by black with 3 times
/// the length and so on.
///
/// This struct is meant to operate on a single line, with the first value in the line given to
/// `CapStoneFinder::new` and any following values given to `CapStoneFinder::advance`
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct LineScanner {
    lookbehind_buf: [usize; 5],
    last_color: PixelColor,
    run_length: usize,
    color_changes: usize,
    current_position: usize,
}

impl LineScanner {
    /// Initialize a new CapStoneFinder with the value of the first pixel in a line
    fn new(initial_col: PixelColor) -> Self {
        LineScanner {
            lookbehind_buf: [0; 5],
            last_color: initial_col,
            run_length: 1,
            color_changes: 0,
            current_position: 0,
        }
    }

    /// Advance the position of the finder with the given color.
    ///
    /// This will return `None` if no pattern matching a CapStone was recently observed. It will
    /// return `Some(position)` if the last added pixel completes a 1:1:3:1:1 pattern of
    /// black/white runs. This is a candidate for capstones.
    fn advance(&mut self, color: PixelColor) -> Option<LinePosition> {
        self.current_position += 1;

        // If we did not observe a color change, we have not reached the boundary of a capstone
        if self.last_color == color {
            self.run_length += 1;
            return None;
        }

        self.last_color = color;
        self.lookbehind_buf.rotate_left(1);
        self.lookbehind_buf[4] = self.run_length;
        self.run_length = 1;
        self.color_changes += 1;

        if self.test_for_capstone() {
            Some(LinePosition {
                left: self.current_position - self.lookbehind_buf.iter().sum::<usize>(),
                stone: self.current_position - self.lookbehind_buf[2..].iter().sum::<usize>(),
                right: self.current_position - self.lookbehind_buf[4],
            })
        } else {
            None
        }
    }

    /// Test if the observed pattern matches that of a capstone.
    ///
    /// Capstones have a distinct pattern of 1:1:3:1:1 of black->white->black->white->black
    /// transitions.
    fn test_for_capstone(&self) -> bool {
        // A capstone should look like > x xxx x < so we have to check after 5 color changes
        // and only if the newly observed color is white
        if PixelColor::White == self.last_color && self.color_changes >= 5 {
            const CHECK: [usize; 5] = [1, 1, 3, 1, 1];
            let avg = (self.lookbehind_buf[0] + self.lookbehind_buf[1] + self.lookbehind_buf[3] + self.lookbehind_buf[4]) / 4;
            let err = avg * 3 / 4;
            for i in 0..5 {
                if self.lookbehind_buf[i] < CHECK[i] * avg - err || self.lookbehind_buf[i] > CHECK[i] * avg + err {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

/// Determine if the given position is an unobserved capstone
///
/// This checks for a few things:
/// * The `left` and `right` positions are connected
/// * The `stone` position is **not** connected to the others
/// * The positions are unclaimed, i.e. not used for other capstones etc.
/// * The ratio between the size of `stone` position and the outer `ring` position is roughly 37.5%
///
/// Returns `true` if all of the above are true, `false` otherwise
fn is_capstone<S>(img: &mut PreparedImage<S>, linepos: &LinePosition, y: usize) -> bool where S: ImageBuffer {
    let ring_reg = img.get_region((linepos.right, y));
    let stone_reg = img.get_region((linepos.stone, y));

    if img.get_pixel_at(linepos.left, y) != img.get_pixel_at(linepos.right, y) {
        return false;
    }

    match (ring_reg, stone_reg) {
        (
            ColoredRegion::Unclaimed {
                color: ring_color,
                pixel_count: ring_count,
                ..
            },
            ColoredRegion::Unclaimed {
                color: stone_color,
                pixel_count: stone_count,
                ..
            }
        ) => {
            let ratio = stone_count * 100 / ring_count;
            // Verify that left is connected to right, and that stone is not connected
            // Also that the pixel counts roughly repsect the 37.5% ratio
            ring_color
                != stone_color && 10 < ratio && ratio < 70
        }
        _ => false,
    }
}

/// Create a capstone at the given position
///
/// * This determines the extend and perspective of the capstone at the given position
/// * It marks the `ring` and `stone` of the capstone as reserved so that it may not be detected
///   again
///
/// Returns the `CapStone` at the given position
fn create_capstone<S>(
    img: &mut PreparedImage<S>,
    linepos: &LinePosition,
    y: usize,
) -> CapStone  where S: ImageBuffer {
    /* Find the corners of the ring */
    let start_point = Point { x: linepos.right as i32, y: y as i32 };
    let first_corner_finder = FirstCornerFinder::new(start_point);
    let first_corner_finder = img.repaint_and_apply((linepos.right, y), PixelColor::Tmp1, first_corner_finder);
    let all_corner_finder = AllCornerFinder::new(start_point, first_corner_finder.best());
    let all_corner_finder = img.repaint_and_apply((linepos.right, y), PixelColor::CapStone, all_corner_finder);
    let corners = all_corner_finder.best();

    /* Set up the perspective transform and find the center */
    let c = Perspective::create(
        &corners,
        7.0,
        7.0,
    );
    let center = c.map(3.5, 3.5);

    CapStone {
        c,
        corners,
        center,
    }
}

/// Find the a corner of a sheared rectangle.
///
/// This finds the point that is the farthest from a given reference point on the rectangle.
/// This point must be one corner
#[derive(Debug, Eq, PartialEq, Clone)]
struct FirstCornerFinder {
    initial: Point,
    best: Point,
    score: i32,
}

impl FirstCornerFinder {
    pub fn new(initial: Point) -> Self {
        FirstCornerFinder {
            initial,
            best: Default::default(),
            score: -1,
        }
    }

    pub fn best(self) -> Point {
        self.best
    }
}

impl AreaFiller for FirstCornerFinder {
    fn update(&mut self, row: Row) {
        let dy = (row.y as i32) - self.initial.y;
        let l_dx = (row.left as i32) - self.initial.x;
        let r_dx = (row.right as i32) - self.initial.x;

        let l_dist = l_dx * l_dx + dy * dy;
        let r_dist = r_dx * r_dx + dy * dy;

        if l_dist > self.score {
            self.score = l_dist;
            self.best = Point {
                x: row.left as i32,
                y: row.y as i32,
            }
        }

        if r_dist > self.score {
            self.score = r_dist;
            self.best = Point {
                x: row.right as i32,
                y: row.y as i32,
            }
        }
    }
}

/// Find the 4 corners of a rectangle
///
/// Expects an initial point in the rectangle as well as a known corner
///
/// The other corners are found by searching extreme points based on the line between initial
/// and corner point. The opposite corner must one of the points that lie the farthest in the
/// opposite direction. The 2 other corners are those that are the farthest left and right from
/// the reference line.
#[derive(Debug, Eq, PartialEq, Clone)]
struct AllCornerFinder {
    baseline: Point,
    best: [Point; 4],
    scores: [i32; 4],
}

impl AllCornerFinder {
    pub fn new(initial: Point, corner: Point) -> Self {
        let baseline = Point {
            x: corner.x - initial.x,
            y: corner.y - initial.y,
        };

        let parallel_score = initial.x * baseline.x + initial.y * baseline.y;
        let orthogonal_score = -initial.x * baseline.y + initial.y * baseline.x;

        AllCornerFinder {
            baseline,
            best: [initial; 4],
            scores: [parallel_score, orthogonal_score, -parallel_score, -orthogonal_score],
        }
    }

    pub fn best(self) -> [Point; 4] {
        self.best
    }
}

impl AreaFiller for AllCornerFinder {
    fn update(&mut self, row: Row) {
        let l_par_score = (row.left as i32) * self.baseline.x + (row.y as i32) * self.baseline.y;
        let l_ort_score = -(row.left as i32) * self.baseline.y + (row.y as i32) * self.baseline.x;
        let l_scores = [l_par_score, l_ort_score, -l_par_score, -l_ort_score];

        let r_par_score = (row.right as i32) * self.baseline.x + (row.y as i32) * self.baseline.y;
        let r_ort_score = -(row.right as i32) * self.baseline.y + (row.y as i32) * self.baseline.x;
        let r_scores = [r_par_score, r_ort_score, -r_par_score, -r_ort_score];

        for j in 0..4 {
            if l_scores[j] > self.scores[j] {
                self.scores[j] = l_scores[j];
                self.best[j] = Point {
                    x: row.left as i32,
                    y: row.y as i32,
                }
            }

            if r_scores[j] > self.scores[j] {
                self.scores[j] = r_scores[j];
                self.best[j] = Point {
                    x: row.right as i32,
                    y: row.y as i32,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepare::BasicImageBuffer;

    #[test]
    fn test_capstone_finder_small() {
        let mut line = [1, 0, 1, 1, 1, 0, 1, 0].iter();

        let mut finder = LineScanner::new(PixelColor::White);
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(Some(LinePosition {
            left: 1,
            stone: 3,
            right: 7,
        }), finder.advance(PixelColor::from(*line.next().unwrap())));
    }

    #[test]
    fn test_capstone_finder_start() {
        let mut line = [0, 1, 1, 1, 0, 1, 0].iter();

        let mut finder = LineScanner::new(PixelColor::Black);
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(Some(LinePosition {
            left: 0,
            stone: 2,
            right: 6,
        }), finder.advance(PixelColor::from(*line.next().unwrap())));
    }

    #[test]
    fn test_capstone_finder_multiple() {
        let mut line = [0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0].iter();

        let mut finder = LineScanner::new(PixelColor::Black);
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(Some(LinePosition {
            left: 0,
            stone: 2,
            right: 6,
        }), finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(Some(LinePosition {
            left: 6,
            stone: 8,
            right: 12,
        }), finder.advance(PixelColor::from(*line.next().unwrap())));
    }

    #[test]
    fn test_capstone_finder_variance() {
        let mut line = [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0].iter();

        let mut finder = LineScanner::new(PixelColor::White);
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(None, finder.advance(PixelColor::from(*line.next().unwrap())));
        assert_eq!(Some(LinePosition {
            left: 1,
            stone: 6,
            right: 13,
        }), finder.advance(PixelColor::from(*line.next().unwrap())));
    }

    fn img_from_array(array: [[u8; 3]; 3]) -> PreparedImage<BasicImageBuffer> {
        crate::PreparedImage::prepare_from_bitmap(3, 3, |x, y| {
            array[y][x] == 1
        })
    }

    #[test]
    fn test_one_corner_finder() {
        let mut test_u = img_from_array([
            [1, 0, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);
        let finder = FirstCornerFinder::new(Point {
            x: 0,
            y: 0,
        });

        let res = test_u.repaint_and_apply((0, 0), PixelColor::Tmp1, finder);
        assert_eq!(Point {
            x: 2,
            y: 2,
        }, res.best());
    }

    #[test]
    fn test_all_corner_finder() {
        let mut test_u = img_from_array([
            [1, 0, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);
        let initial = Point {
            x: 0,
            y: 0,
        };
        let one_corner = Point {
            x: 2,
            y: 2,
        };
        let finder = AllCornerFinder::new(initial, one_corner);

        let res = test_u.repaint_and_apply((0, 0), PixelColor::Tmp1, finder);
        let corners = res.best();
        assert_eq!(Point {
            x: 2,
            y: 2,
        }, corners[0]);
        assert_eq!(Point {
            x: 0,
            y: 2,
        }, corners[1]);
        assert_eq!(Point {
            x: 0,
            y: 0,
        }, corners[2]);
        assert_eq!(Point {
            x: 2,
            y: 0,
        }, corners[3]);
    }

    fn load_and_find(img: &[u8]) -> Vec<CapStone> {
        let img = image::load_from_memory(img).unwrap().to_luma();
        let w = img.width() as usize;
        let h = img.height() as usize;
        let mut img = crate::PreparedImage::prepare_from_greyscale(w, h, |x, y| {
            img.get_pixel(x as u32, y as u32).data[0]
        });
        crate::capstones_from_image(&mut img)
    }


    #[test]
    fn test_cap() {
        let caps = load_and_find(include_bytes!("test_data/cap.png"));
        assert_eq!(1, caps.len());
    }

    #[test]
    fn test_cap_connected() {
        let caps = load_and_find(include_bytes!("test_data/cap_connect.png"));
        assert_eq!(0, caps.len());
    }

    #[test]
    fn test_cap_disconnected() {
        let caps = load_and_find(include_bytes!("test_data/cap_disconnect.png"));
        assert_eq!(0, caps.len());
    }

    #[test]
    fn test_cap_size() {
        let caps = load_and_find(include_bytes!("test_data/cap_size.png"));
        assert_eq!(0, caps.len());
    }

}
