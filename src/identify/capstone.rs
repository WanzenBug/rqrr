#[cfg(feature = "debug-plot")]
use gnuplot;

use crate::Image;
use crate::identify::{Point};

use super::PixelColor;
use crate::identify::helper::Perspective;

#[derive(Debug, Clone)]
pub struct CapStone {
    pub corners: [Point; 4],
    pub center: Point,
    pub c: Perspective,
}

#[derive(Debug, Clone)]
pub struct PolygonScoreData {
    pub ref_0: Point,
    pub scores: [i32; 4],
    pub corners: [Point; 4],
}


impl CapStone {
    #[cfg(feature = "debug-plot")]
    pub fn plot(&self, fig: &mut gnuplot::Axes2D) {
        fig.lines(self.corners.iter()
                      .cycle()
                      .take(5)
                      .map(|p| p.x),
                  self.corners.iter()
                      .cycle()
                      .take(5)
                      .map(|p| p.y),
                  &[gnuplot::Color("green"), gnuplot::LineWidth(4.0)]);
    }
}

pub fn capstones_from_image_with_debug<F, G>(img: &mut Image, mut debug: F, mut cap_debug: G) -> Vec<CapStone> where F: FnMut(&Image, &str, ::std::ops::RangeInclusive<usize>, usize), G: FnMut(::std::ops::RangeInclusive<usize>, usize, &PolygonScoreData, &str) {
    let mut res = Vec::new();

    for y in 0..img.h {
        let mut finder = CapStoneFinder::new(img[(0, y)]);
        for x in 1..img.w {
            if finder.check_for_capstone(img[(x, y)]) {
                debug(img, "maybe cap", x..=x, y);
                let linepos = finder.get_positions(x);
                if is_capstone(img, &linepos, y, &mut debug) {
                    let cap = create_capstone_debug(img, &linepos, y, &mut cap_debug);
                    debug(img, "is cap", x..=x, y);
                    res.push(cap);
                }
            }
            debug(img, "pixel done", x..=x, y)
        }
        debug(img, "line done", 0..=img.w-1, y)
    }

    res
}

pub fn capstones_from_image(img: &mut Image) -> Vec<CapStone> {
    capstones_from_image_with_debug(img, |_, _, _, _| (), |_, _, _,_| ())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct LinePosition {
    left: usize,
    stone: usize,
    right: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct CapStoneFinder {
    lookbehind_buf: [usize; 5],
    last_color: PixelColor,
    run_length: usize,
    color_changes: usize,
}

fn looks_like_capstone(lookbehind_buf: &[usize; 5]) -> bool {
    const CHECK: [usize; 5] = [1, 1, 3, 1, 1];
    let avg = (lookbehind_buf[0] + lookbehind_buf[1] + lookbehind_buf[3] + lookbehind_buf[4]) / 4;
    let err = avg * 3 / 4;
    for i in 0..5 {
        if lookbehind_buf[i] < CHECK[i] * avg - err || lookbehind_buf[i] > CHECK[i] * avg + err {
            return false;
        }
    }
    true
}

impl CapStoneFinder {
    fn new(initial_col: PixelColor) -> Self {
        CapStoneFinder {
            lookbehind_buf: [0; 5],
            last_color: initial_col,
            run_length: 1,
            color_changes: 0,
        }
    }

    fn get_positions(&self, last_loc: usize) -> LinePosition {
        LinePosition {
            left: last_loc - self.lookbehind_buf.iter().sum::<usize>(),
            stone: last_loc - self.lookbehind_buf[2..].iter().sum::<usize>(),
            right: last_loc - self.lookbehind_buf[4],
        }
    }

    fn check_for_capstone(&mut self, color: PixelColor) -> bool {
        let mut ret = false;

        if self.last_color != color {
            self.lookbehind_buf.rotate_left(1);
            self.lookbehind_buf[4] = self.run_length;
            self.run_length = 0;
            self.color_changes += 1;


            if color == PixelColor::White && self.color_changes >= 5 {
                ret = looks_like_capstone(&self.lookbehind_buf);
            }
        }

        self.run_length += 1;
        self.last_color = color;

        ret
    }
}

fn is_capstone<F>(
    img: &mut Image,
    linepos: &LinePosition,
    y: usize,
    mut debug: F
) -> bool where F: FnMut(&Image, &str, ::std::ops::RangeInclusive<usize>, usize) {
    if img[(linepos.left, y)] != img[(linepos.right, y)] || img[(linepos.right, y)] != PixelColor::Black {
        return false;
    }

    debug(img, "paint ring", linepos.left..=linepos.right, y);
    let (old_ring, ring_count) = img.repaint_and_count((linepos.right, y), PixelColor::CheckCapstone, &mut debug);

    // Verify that left is connected to right, and that stone is not connected
    if img[(linepos.left, y)] != PixelColor::CheckCapstone || img[(linepos.stone, y)] == PixelColor::CheckCapstone {
        debug(img, "invalid connect", linepos.left..=linepos.right, y);
        img.repaint_and_count((linepos.right, y), old_ring, &mut debug);
        return false;
    }

    debug(img, "paint stone", linepos.left..=linepos.right, y);
    let (old_stone, stone_count) = img.repaint_and_count((linepos.stone, y), PixelColor::CheckCapstone, &mut debug);

    /* Ratio should ideally be 37.5 */
    let ratio = stone_count * 100 / ring_count;
    if ratio < 10 || ratio > 70 {
        debug(img, "invalid count", linepos.left..=linepos.right, y);
        img.repaint_and_count((linepos.right, y), old_ring, &mut debug);
        img.repaint_and_count((linepos.stone, y), old_stone, &mut debug);
        return false;
    }
    debug(img, "is_cap", linepos.left..=linepos.right, y);
    true
}

fn create_capstone(
    img: &mut Image,
    linepos: &LinePosition,
    y: usize,
) -> CapStone {
    create_capstone_debug(img, linepos, y, |_, _, _, _| ())
}

fn create_capstone_debug<F>(
    img: &mut Image,
    linepos: &LinePosition,
    y: usize,
    debug: F
) -> CapStone where F: FnMut(::std::ops::RangeInclusive<usize>, usize, &PolygonScoreData, &str) {
    /* Find the corners of the ring */
    let corners = find_region_corners_debug(img, linepos.right, y, debug);
    img.repaint_and_count((linepos.stone, y), PixelColor::CapStone, |_, _, _, _| ());

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

fn  find_region_corners(
    img: &mut Image,
    x: usize,
    y: usize,
) {
    find_region_corners_debug(img, x, y, |_ ,_, _, _|());
}

fn  find_region_corners_debug<F>(
    img: &mut Image,
    x: usize,
    y: usize,
    mut debug: F
) -> [Point; 4] where F: FnMut(::std::ops::RangeInclusive<usize>, usize, &PolygonScoreData, &str) {
    let ix = x as i32;
    let iy = y as i32;

    let ref_0 = Point {
        x: ix,
        y: iy,
    };

    let mut psd = PolygonScoreData {
        ref_0: ref_0.clone(),
        scores: [
            -1,
            0,
            0,
            0,
        ],
        corners: [
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        ],
    };
    img.flood_fill(x, y, PixelColor::CheckCapstone, PixelColor::FindOneCorner, &mut |_, row| {
        find_one_corner(&mut psd, row.y, row.left, row.right);
        debug(row.left..=row.right, row.y, &psd, "one");
    });
    psd.ref_0.x = psd.corners[0].x - psd.ref_0.x;
    psd.ref_0.y = psd.corners[0].y - psd.ref_0.y;

    psd.corners = [
        ref_0.clone(),
        ref_0.clone(),
        ref_0.clone(),
        ref_0.clone(),
    ];

    let i = ix * psd.ref_0.x + iy * psd.ref_0.y;
    psd.scores[0] = i;
    psd.scores[2] = -i;
    let i = ix * -psd.ref_0.y + iy * psd.ref_0.x;
    psd.scores[1] = i;
    psd.scores[3] = -i;
    // Recolor to expected color right here
    img.flood_fill(x, y, PixelColor::FindOneCorner, PixelColor::CapStone, &mut |_, row| {
        find_other_corners(&mut psd, row.y, row.left, row.right);
        debug(row.left..=row.right, row.y, &psd, "rest");
    });

    let PolygonScoreData { corners, .. } = psd;
    corners
}


fn find_one_corner(
    psd: &mut PolygonScoreData,
    y: usize,
    left: usize,
    right: usize,
) -> () {
    let y = y as i32;
    let xs = [left as i32, right as i32];
    let dy = y - psd.ref_0.y;
    for i in 0..2 {
        let dx = xs[i] - psd.ref_0.x;
        let d = dx * dx + dy * dy;
        if d > psd.scores[0] {
            psd.scores[0] = d;
            psd.corners[0].x = xs[i];
            psd.corners[0].y = y
        }
    }
}

fn find_other_corners(
    psd: &mut PolygonScoreData,
    y: usize,
    left: usize,
    right: usize,
) -> () {
    let y = y as i32;
    let xs = [left as i32, right as i32];
    for i in 0..2 {
        let up = xs[i] * psd.ref_0.x + y * psd.ref_0.y;
        let right_0 = xs[i] * -psd.ref_0.y + y * psd.ref_0.x;
        let scores = [up, right_0, -up, -right_0];
        for j in 0..4 {
            if scores[j] > psd.scores[j] {
                psd.scores[j] = scores[j];
                psd.corners[j].x = xs[i];
                psd.corners[j].y = y;
            }
        }
    }
}

