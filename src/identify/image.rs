use std::cmp;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

#[cfg(feature = "debug-plot")]
use gnuplot::Axes2D;

use crate::identify::Point;


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Row {
    pub left: usize,
    pub right: usize,
    pub y: usize,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PixelColor {
    Black,
    White,
    CapStone,
    FindAlignment,
    Alignment,
    CheckCapstone,
    FindOneCorner,
    TimingBlack,
    TimingWhite,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub pixels: Box<[PixelColor]>,
}

impl Image {
    pub fn from_greyscale<F>(w: usize,
                             h: usize,
                             fill: F, ) -> Self where F: FnMut(usize, usize) -> u8 {
        Image::from_greyscale_debug(w, h, fill, |_, _, _| ())
    }

    pub fn from_greyscale_debug<F, D>(w: usize,
                                      h: usize,
                                      mut fill: F,
                                      mut debug: D,
    ) -> Self where F: FnMut(usize, usize) -> u8, D: FnMut(usize, usize, &[PixelColor]) -> () {
        let capacity = w.checked_mul(h).expect("Image dimensions caused overflow");
        let mut data = Vec::with_capacity(capacity);

        let mut row_average = vec![0; w];
        let mut avg_v = 0;
        let mut avg_u = 0;

        let threshold_s = cmp::max(w / 8, 1);

        for y in 0..h {
            for r in &mut row_average {
                *r = 0;
            }

            for x in 0..w {
                let (v, u) = if y % 2 == 0 {
                    (w - 1 - x, x)
                } else {
                    (x, w - 1 - x)
                };
                avg_v = avg_v * (threshold_s - 1) / threshold_s + fill(v, y) as usize;
                avg_u = avg_u * (threshold_s - 1) / threshold_s + fill(u, y) as usize;
                row_average[v] += avg_v;
                row_average[u] += avg_u;
            }

            for x in 0..w {
                let fill = if (fill(x, y) as usize) < row_average[x] * (100 - 5) / (200 * threshold_s) {
                    PixelColor::Black
                } else {
                    PixelColor::White
                };
                data.push(fill);
            }
            debug(w, y + 1, &data)
        }

        let pixels = data.into_boxed_slice();
        Image {
            w,
            h,
            pixels,
        }
    }

    pub(crate) fn repaint_and_count(&mut self, (x, y): (usize, usize), target_color: PixelColor) -> (PixelColor, usize) {
        let src_color = self[(x, y)];

        if src_color == target_color {
            panic!("Tried to count already counted region!");
        }

        let mut count = 0;
        {
            let count_ref = &mut count;
            self.flood_fill(x, y, src_color, target_color, &mut |img, row| {
                *count_ref += row.right - row.left + 1;
            });
        }
        (src_color, count)
    }


    pub(crate) fn flood_fill(
        &mut self,
        x: usize,
        y: usize,
        from: PixelColor,
        to: PixelColor,
        func: &mut FnMut(&Image, Row) -> (),
    ) {
        assert_ne!(from, to);

        let w = self.w;
        let mut queue = Vec::new();
        queue.push((x, y));
        while !queue.is_empty() {
            let (x, y) = queue.pop().expect("Just checked, queue is not empty");

            // Bail early in case there is nothing to fill
            if self[(x, y)] == to || self[(x, y)] != from {
                continue;
            }

            let mut left = x;
            let mut right = x;
            {
                let row = &mut self[(0..w, y)];
                while left > 0 && row[left - 1] == from {
                    left -= 1;
                }
                while right < w - 1 && row[right + 1] == from {
                    right += 1
                }

                /* Fill the extent */
                for p in &mut row[left..right + 1] {
                    *p = to;
                }
            }

            func(self, Row {
                left,
                right,
                y,
            });

            /* Seed new flood-fills */
            if y > 0 {
                let mut seeded_previous = false;
                for x in left..=right {
                    let p = self[(x, y - 1)];
                    if p == from {
                        if !seeded_previous {
                            queue.push((x, y - 1));
                        }
                        seeded_previous = true;
                    } else {
                        seeded_previous = false;
                    }
                }
            }
            if y < self.h - 1 {
                let mut seeded_previous = false;
                for x in left..=right {
                    let p = self[(x, y + 1)];
                    if p == from {
                        if !seeded_previous {
                            queue.push((x, y + 1));
                        }
                        seeded_previous = true;
                    } else {
                        seeded_previous = false;
                    }
                }
            }
        }
    }


    #[cfg(feature = "debug-plot")]
    pub fn plot<'a, 'b>(&'a self, axes: &'b mut Axes2D) -> &'b mut Axes2D {
        axes.image(self.pixels.iter().map(|b| match *b {
            PixelColor::White => 0,
            PixelColor::Black => 1,
            PixelColor::TimingWhite => 4,
            PixelColor::TimingBlack => 5,
            PixelColor::CapStone => 10,
            PixelColor::Alignment => 11,
            PixelColor::FindOneCorner => 20,
            PixelColor::CheckCapstone => 21,
            PixelColor::FindAlignment => 22,
        }), self.h, self.w, Some((0.0, 0.0, self.w as f64, self.h as f64)), &[])
    }
}


impl Index<(Range<usize>, usize)> for Image {
    type Output = [PixelColor];

    fn index(&self, (xs, y): (Range<usize>, usize)) -> &<Self as Index<(Range<usize>, usize)>>::Output {
        let start = y * self.w + xs.start;
        let end = y * self.w + xs.end;
        &self.pixels[start..end]
    }
}

impl IndexMut<(Range<usize>, usize)> for Image {
    fn index_mut(&mut self, (xs, y): (Range<usize>, usize)) -> &mut <Self as Index<(Range<usize>, usize)>>::Output {
        let start = y * self.w + xs.start;
        let end = y * self.w + xs.end;
        &mut self.pixels[start..end]
    }
}

impl Index<Point> for Image {
    type Output = PixelColor;

    fn index(&self, index: Point) -> &<Self as Index<Point>>::Output {
        assert!(index.x >= 0);
        assert!((index.x as usize) < self.w);
        assert!(index.y >= 0);
        assert!((index.y as usize) < self.h);

        &self[(index.x as usize, index.y as usize)]
    }
}

impl IndexMut<Point> for Image {
    fn index_mut(&mut self, index: Point) -> &mut <Self as Index<Point>>::Output {
        assert!(index.x >= 0);
        assert!((index.x as usize) < self.w);
        assert!(index.y >= 0);
        assert!((index.y as usize) < self.h);

        &mut self[(index.x as usize, index.y as usize)]
    }
}


impl Index<(usize, usize)> for Image {
    type Output = PixelColor;

    fn index(&self, (x, y): (usize, usize)) -> &<Self as Index<(usize, usize)>>::Output {
        &self.pixels[y * self.w + x]
    }
}

impl IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut <Self as Index<(usize, usize)>>::Output {
        &mut self.pixels[y * self.w + x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn img_from_array(array: [[u8; 3]; 3]) -> Image {
        let mut pixels = Vec::new();
        for col in array.iter() {
            for item in col.iter() {
                if *item == 0 {
                    pixels.push(PixelColor::White)
                } else {
                    pixels.push(PixelColor::Black)
                }
            }
        }

        Image {
            w: 3,
            h: 3,
            pixels: pixels.into_boxed_slice(),
        }
    }

    #[test]
    fn test_flood_fill_full() {
        let mut test_full = img_from_array([
            [1, 1, 1],
            [1, 1, 1],
            [1, 1, 1],
        ]);

        test_full.flood_fill(0, 0, PixelColor::Black, PixelColor::CapStone, &mut |_, _| ());

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(test_full[(x, y)], PixelColor::CapStone);
            }
        }
    }

    #[test]
    fn test_flood_fill_single() {
        let mut test_single = img_from_array([
            [1, 0, 1],
            [0, 1, 0],
            [1, 0, 1],
        ]);

        test_single.flood_fill(1, 1, PixelColor::Black, PixelColor::CapStone, &mut |_, _| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && y == 1 {
                    assert_eq!(test_single[(x, y)], PixelColor::CapStone);
                } else {
                    let col = if (x + y) % 2 == 0 {
                        PixelColor::Black
                    } else {
                        PixelColor::White
                    };
                    assert_eq!(test_single[(x, y)], col);
                }
            }
        }
    }

    #[test]
    fn test_flood_fill_ring() {
        let mut test_ring = img_from_array([
            [1, 1, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);

        test_ring.flood_fill(0, 0, PixelColor::Black, PixelColor::CapStone, &mut |_, _| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && y == 1 {
                    assert_eq!(test_ring[(x, y)], PixelColor::White);
                } else {
                    assert_eq!(test_ring[(x, y)], PixelColor::CapStone);
                }
            }
        }
    }

    #[test]
    fn test_flood_fill_u() {
        let mut test_u = img_from_array([
            [1, 0, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);


        test_u.flood_fill(0, 0, PixelColor::Black, PixelColor::CapStone, &mut |_, _| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && (y == 0 || y == 1) {
                    assert_eq!(test_u[(x, y)], PixelColor::White);
                } else {
                    assert_eq!(test_u[(x, y)], PixelColor::CapStone);
                }
            }
        }
    }

    #[test]
    fn test_flood_fill_empty() {
        let mut test_empty = img_from_array([
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
        ]);

        test_empty.flood_fill(1, 1, PixelColor::Black, PixelColor::CapStone, &mut |_, _| ());

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(test_empty[(x, y)], PixelColor::White)
            }
        }
    }

    #[test]
    fn test_repaint_and_count() {
        let mut test_u = img_from_array([
            [1, 0, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);

        let (old, c) = test_u.repaint_and_count((0, 0), PixelColor::CapStone);
        assert_eq!(c, 7);
        assert_eq!(old, PixelColor::Black);
        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && (y == 0 || y == 1) {
                    assert_eq!(test_u[(x, y)], PixelColor::White);
                } else {
                    assert_eq!(test_u[(x, y)], PixelColor::CapStone);
                }
            }
        }
    }
}
