use std::{
    cmp,
    mem,
};

use crate::{
    CapStone,
    GridImage,
    identify::{
        match_capstones::CapStoneGroup,
        image::Region,
    },
    Image,
    version_db::VERSION_DATA_BASE,
};

use super::{
    helper,
    PixelColor,
    Point,
};
use crate::identify::image::{AreaFiller, Row};

#[derive(Debug, Clone)]
pub struct Grid {
    pub caps: [CapStone; 3],
    pub align: Point,
    pub grid_size: usize,
    pub c: helper::Perspective,
}

impl Grid {
    pub fn from_group(img: &mut Image, mut group: CapStoneGroup) -> Option<Self> {
        /* Construct the hypotenuse line from A to C. B should be to
         * the left of this line.
         */
        let mut h0 = group.0.center;
        let mut hd = Point {
            x: group.2.center.x - group.0.center.x,
            y: group.2.center.y - group.0.center.y,
        };

        /* Make sure A-B-C is clockwise */
        if (group.1.center.x - h0.x) * -hd.y + (group.1.center.y - h0.y) * hd.x > 0 {
            mem::swap(&mut group.0, &mut group.2);
            hd.x = -hd.x;
            hd.y = -hd.y
        }

        /* Rotate each capstone so that corner 0 is top-left with respect
         * to the grid.
         */
        rotate_capstone(&mut group.0, &mut h0, &mut hd);
        rotate_capstone(&mut group.1, &mut h0, &mut hd);
        rotate_capstone(&mut group.2, &mut h0, &mut hd);

        /* Check the timing pattern. This doesn't require a perspective
         * transform.
         */
        let grid_size = measure_timing_pattern(img, &group);

        /* Make an estimate based for the alignment pattern based on extending
         * lines from capstones A and C.
         */

        let mut align = helper::line_intersect(
            &group.0.corners[0],
            &group.0.corners[1],
            &group.2.corners[0],
            &group.2.corners[3],
        )?;

        /* On V2+ grids, we should use the alignment pattern. */
        if grid_size > 21 {
            /* Try to find the actual location of the alignment pattern. */
            align = find_alignment_pattern(img, align, &group.0, &group.2)?;
            let score = -hd.y * align.x + hd.x * align.y;
            let finder = LeftMostFinder {
                line_p: hd,
                best: align,
                score,
            };
            let found = img.repaint_and_apply((align.x as usize, align.y as usize),PixelColor::Alignment, finder);
            align = found.best;
        }

        let c = setup_perspective(img, &group, align, grid_size);
        let caps = [group.0, group.1, group.2];

        Some(Grid {
            align,
            caps,
            grid_size,
            c,
        })
    }

    pub fn into_grid_image(self, img: &Image) -> RefGridImage {
        RefGridImage {
            grid: self,
            img,
        }
    }
}

pub struct RefGridImage<'a> {
    grid: Grid,
    img: &'a Image,
}

impl<'a> GridImage for RefGridImage<'a> {
    fn size(&self) -> usize {
        self.grid.grid_size
    }

    fn bit(&self, y: usize, x: usize) -> bool {
        let p = self.grid.c.map(x as f64 + 0.5, y as f64 + 0.5);
        PixelColor::White != self.img[p]
    }
}

impl<'a> RefGridImage<'a> {
    #[cfg(feature = "debug-plot")]
    pub fn plot<'b>(&self, ax: &'b mut gnuplot::Axes2D) -> &'b mut gnuplot::Axes2D {
        for x in 0..=self.grid.grid_size {
            let from = self.grid.c.map(x as f64, 0.0);
            let to = self.grid.c.map(x as f64, self.grid.grid_size as f64);
            ax.lines(&[from.x, to.x, ], &[from.y, to.y, ], &[gnuplot::Color("white")]);
        }

        for y in 0..=self.grid.grid_size {
            let from = self.grid.c.map(0.0, y as f64);
            let to = self.grid.c.map(self.grid.grid_size as f64, y as f64);
            ax.lines(&[from.x, to.x, ], &[from.y, to.y, ], &[gnuplot::Color("white")]);
        }

        ax
    }
}


fn setup_perspective(img: &Image, caps: &CapStoneGroup, align: Point, grid_size: usize) -> helper::Perspective {
    let inital = helper::Perspective::create(&[
        caps.1.corners[0],
        caps.2.corners[0],
        align,
        caps.0.corners[0],
    ], (grid_size - 7) as f64, (grid_size - 7) as f64);

    jiggle_perspective(img, inital, grid_size)
}

fn rotate_capstone(
    cap: &mut CapStone,
    h0: &Point,
    hd: &Point,
) -> () {
    let (best_idx, _) = cap.corners.iter()
        .enumerate()
        .min_by_key(|(_, a)| {
            (a.x - h0.x) * (-hd.y) + (a.y - h0.y) * hd.x
        }).expect("corners cannot be empty");

    /* Rotate the capstone */
    cap.corners.rotate_left(best_idx);
    cap.c = helper::Perspective::create(&cap.corners, 7.0, 7.0);
}

//* Try the measure the timing pattern for a given QR code. This does
// * not require the global perspective to have been set up, but it
// * does require that the capstone corners have been set to their
// * canonical rotation.
// *
// * For each capstone, we find a point in the middle of the ring band
// * which is nearest the centre of the code. Using these points, we do
// * a horizontal and a vertical timing scan.
// */
fn measure_timing_pattern(
    img: &Image,
    caps: &CapStoneGroup,
) -> usize {
    const US: [f64; 3] = [6.5f64, 6.5f64, 0.5f64];
    const VS: [f64; 3] = [0.5f64, 6.5f64, 6.5f64];
    let tpet0 = caps.0.c.map(US[0], VS[0]);
    let tpet1 = caps.1.c.map(US[1], VS[1]);
    let tpet2 = caps.2.c.map(US[2], VS[2]);

    let hscan = timing_scan(img, &tpet1, &tpet2);
    let vscan = timing_scan(img, &tpet1, &tpet0);

    let scan = cmp::max(hscan, vscan);

    /* Choose the nearest allowable grid size */
    assert!(scan >= 1);
    let size = scan * 2 + 13;
    let ver = (size as f64 - 15.0).floor() as usize / 4;
    ver * 4 + 17
}

fn timing_scan(
    img: &Image,
    p0: &Point,
    p1: &Point,
) -> usize {
    let mut run_length = 0;
    let mut count = 0;
    for p in helper::BresenhamScan::new(p0, p1) {
        let pixel = img[p];
        if PixelColor::White != pixel {
            if run_length >= 2 {
                count += 1
            }
            run_length = 0;
        } else {
            run_length += 1;
        }
    }

    count
}


fn find_alignment_pattern(img: &mut Image, mut align_seed: Point, c0: &CapStone, c2: &CapStone) -> Option<Point> {
    /* Guess another two corners of the alignment pattern so that we
     * can estimate its size.
     */
    let (u, v) = c0.c.unmap(&align_seed);
    let a = c0.c.map(u, v + 1.0);
    let (u, v) = c2.c.unmap(&align_seed);
    let c = c2.c.map(u + 1.0, v);
    let size_estimate = ((a.x - align_seed.x) * -(c.y - align_seed.y) +
        (a.y - align_seed.y) * (c.x - align_seed.x)).abs() as usize;

    /* Spiral outwards from the estimate point until we find something
     * roughly the right size. Don't look too far from the estimate
     * point.
     */
    let mut dir = 0;
    let mut step_size = 1;

    while step_size * step_size < size_estimate * 100 {
        const DX_MAP: [i32; 4] = [1, 0, -1, 0];
        const DY_MAP: [i32; 4] = [0, -1, 0, 1];
        for _pass in 0..step_size {
            let x = align_seed.x as usize;
            let y = align_seed.y as usize;

            // Alignment pattern should not be white
            if PixelColor::White != img[(x, y)] {
                let region = img.get_region((x, y));
                let count = match region {
                    Region::Unclaimed { pixel_count, .. } => { pixel_count }
                    _ => continue,
                };

                // Matches expected size of alignment pattern
                if count >= size_estimate / 2 && count <= size_estimate * 2 {
                    return Some(align_seed);
                }
            }

            align_seed.x += DX_MAP[dir];
            align_seed.y += DY_MAP[dir];
        }

        // Cycle directions
        dir = (dir + 1) % 4;
        if dir & 1 == 0 {
            step_size += 1
        }
    }
    None
}

struct LeftMostFinder {
    line_p: Point,
    best: Point,
    score: i32,
}

impl AreaFiller for LeftMostFinder {
    fn update(&mut self, row: Row) {
        let left_d = -self.line_p.y * (row.left as i32) + self.line_p.x * row.y as i32;
        let right_d = -self.line_p.y * (row.right as i32) + self.line_p.x * row.y as i32;

        if left_d < self.score {
            self.score = left_d;
            self.best.x = row.left as i32;
            self.best.y = row.y as i32;
        }

        if right_d < self.score {
            self.score = right_d;
            self.best.x = row.right as i32;
            self.best.y = row.y as i32;
        }
    }
}


fn jiggle_perspective(img: &Image, mut perspective: helper::Perspective, grid_size: usize) -> helper::Perspective {
    let mut best = fitness_all(img, &perspective, grid_size);
    let mut adjustments: [f64; 8] = [
        perspective.0[0] * 0.02f64,
        perspective.0[1] * 0.02f64,
        perspective.0[2] * 0.02f64,
        perspective.0[3] * 0.02f64,
        perspective.0[4] * 0.02f64,
        perspective.0[5] * 0.02f64,
        perspective.0[6] * 0.02f64,
        perspective.0[7] * 0.02f64
    ];

    for _pass in 0..5 {
        for i in 0..16 {
            let j = i >> 1;
            let old = perspective.0[j];
            let step = adjustments[j];

            let new = if i & 1 != 0 {
                old + step
            } else {
                old - step
            };

            perspective.0[j] = new;
            let test = fitness_all(img, &perspective, grid_size);
            if test > best {
                best = test
            } else {
                perspective.0[j] = old
            }
        }

        for i in 0..8 {
            adjustments[i] *= 0.5f64;
        }
    }
    perspective
}
/* Compute a fitness score for the currently configured perspective
 * transform, using the features we expect to find by scanning the
 * grid.
 */
fn fitness_all(img: &Image, perspective: &helper::Perspective, grid_size: usize) -> i32 {
    let version = (grid_size - 17) / 4;
    let info = &VERSION_DATA_BASE[version];
    let mut score = 0;

    /* Check the timing pattern */
    for i in 0..(grid_size as i32 - 14) {
        let expect = if 0 != i & 1 { 1 } else { -1 };
        score += fitness_cell(img, perspective, i + 7, 6) * expect;
        score += fitness_cell(img, perspective, 6, i + 7) * expect;
    }

    /* Check capstones */
    score += fitness_capstone(img, perspective, 0, 0);
    score += fitness_capstone(img, perspective, grid_size as i32 - 7, 0);
    score += fitness_capstone(img, perspective, 0, grid_size as i32 - 7);

    /* Check alignment patterns */
    let mut ap_count = 0;
    while ap_count < 7 && info.apat[ap_count] != 0 {
        ap_count += 1
    }
    for i in 1..(ap_count.saturating_sub(1)) {
        score += fitness_apat(img, perspective, 6, info.apat[i] as i32);
        score += fitness_apat(img, perspective, info.apat[i] as i32, 6);
    }
    for i in 1..ap_count {
        for j in 1..ap_count {
            score += fitness_apat(img, perspective, info.apat[i] as i32, info.apat[j] as i32);
        }
    }
    score
}

fn fitness_apat(
    img: &Image,
    perspective: &helper::Perspective,
    cx: i32,
    cy: i32,
) -> i32 {
    fitness_cell(img, perspective, cx, cy)
        - fitness_ring(img, perspective, cx, cy, 1)
        + fitness_ring(img, perspective, cx, cy, 2)
}

fn fitness_ring(
    img: &Image,
    perspective: &helper::Perspective,
    cx: i32,
    cy: i32,
    radius: i32,
) -> i32 {
    let mut score = 0;
    for i in 0..(radius * 2) {
        score += fitness_cell(img, perspective, cx - radius + i, cy - radius);
        score += fitness_cell(img, perspective, cx - radius, cy + radius - i);
        score += fitness_cell(img, perspective, cx + radius, cy - radius + i);
        score += fitness_cell(img, perspective, cx + radius - i, cy + radius);
    }
    score
}

fn fitness_cell(
    img: &Image,
    perspective: &helper::Perspective,
    x: i32,
    y: i32,
) -> i32 {
    const OFFSETS: [f64; 3] = [0.3f64, 0.5f64, 0.7f64];
    let mut score = 0;
    for v in 0..3 {
        for u in 0..3 {
            let p = perspective.map(x as f64 + OFFSETS[u], y as f64 + OFFSETS[v]);
            if !(p.y < 0 || p.y as usize >= img.height() || p.x < 0 || p.x as usize >= img.width()) {
                if PixelColor::White != img[p] {
                    score += 1
                } else {
                    score -= 1
                }
            }
        }
    }
    score
}


fn fitness_capstone(
    img: &Image,
    perspective: &helper::Perspective,
    x: i32,
    y: i32,
) -> i32 {
    fitness_cell(img, perspective, x + 3, y + 3)
        + fitness_ring(img, perspective, x + 3, y + 3, 1)
        - fitness_ring(img, perspective, x + 3, y + 3, 2)
        + fitness_ring(img, perspective, x + 3, y + 3, 3)
}
