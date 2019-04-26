use crate::{
    identify::{
        Point,
        image::{Region, Image, Row},
        helper::Perspective,
    }
};

use super::PixelColor;

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

pub fn capstones_from_image(img: &mut Image) -> Vec<CapStone> {
    let mut res = Vec::new();

    for y in 0..img.height() {
        let mut finder = CapStoneFinder::new(img[(0, y)].into());
        for x in 1..img.width() {
            if finder.check_for_capstone(img[(x, y)].into()) {
                let linepos = finder.get_positions(x);
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

fn is_capstone(img: &mut Image, linepos: &LinePosition, y: usize) -> bool {
    let ring_reg = img.get_region((linepos.right, y));
    let stone_reg = img.get_region((linepos.stone, y));

    if img[(linepos.left, y)] != img[(linepos.right, y)] {
        return false;
    }

    match (ring_reg, stone_reg) {
        (
            Region::Unclaimed {
                color: ring_color,
                pixel_count: ring_count,
                ..
            },
            Region::Unclaimed {
                color: stone_color,
                pixel_count: stone_count,
                ..
            }
        ) => {


            let ratio = stone_count * 100 / ring_count;
            // Verify that left is connected to right, and that stone is not connected
            // Also that the pixel counts roughly repsect the 37.5% ratio
            ring_color != stone_color && 10 < ratio && ratio < 70
        }
        _ => false,
    }
}

fn create_capstone(
    img: &mut Image,
    linepos: &LinePosition,
    y: usize,
) -> CapStone {
    /* Find the corners of the ring */
    let start_point = Point { x: linepos.right as i32, y: y as i32 };
    let mut first_corner_finder = FirstCornerFinder::new(start_point);
    img.repaint_and_apply((linepos.right, y), PixelColor::Tmp1, |row| first_corner_finder.update(row));
    let mut all_corner_finder = AllCornerFinder::new(start_point, first_corner_finder.best());
    // Recolor to expected color right here
    img.repaint_and_apply((linepos.right, y), PixelColor::CapStone, |row| all_corner_finder.update(row));
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

    pub fn update(&mut self, row: Row) {
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

    pub fn best(self) -> Point {
        self.best
    }
}

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
            best: [baseline; 4],
            scores: [parallel_score, orthogonal_score, -parallel_score, -orthogonal_score],
        }
    }

    pub fn update(&mut self, row: Row) {
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

    pub fn best(self) -> [Point; 4] {
        self.best
    }
}
