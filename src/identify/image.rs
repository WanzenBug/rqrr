use std::cmp;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

use crate::identify::Point;

/// An black-and-white image that can be mutated on search for QR codes
///
/// During search for QR codes, some black zones will be recolored in 'different' shades of black.
/// This is done to speed up the search and mitigate the impact of a huge zones.
#[derive(Clone)]
pub struct SearchableImage {
    w: usize,
    h: usize,
    pixels: Box<[u8]>,
    unclaimed_regions: [Region; 250],
    unclaimed_count: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Row {
    pub left: usize,
    pub right: usize,
    pub y: usize,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PixelColor {
    White,
    Black,
    CapStone,
    Alignment,
    Tmp1,
    Tmp2,
    Discarded(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Region {
    Unclaimed {
        color: PixelColor,
        src_x: usize,
        src_y: usize,
        pixel_count: usize,
    },
    CapStone,
    Alignment,
    Tmp1,
    Tmp2,
}

impl From<u8> for PixelColor {
    fn from(x: u8) -> Self {
        match x {
            0 => PixelColor::White,
            1 => PixelColor::Black,
            2 => PixelColor::CapStone,
            3 => PixelColor::Alignment,
            4 => PixelColor::Tmp1,
            5 => PixelColor::Tmp2,
            x => PixelColor::Discarded(x - 6),
        }
    }
}

impl From<PixelColor> for u8 {
    fn from(c: PixelColor) -> Self {
        match c {
            PixelColor::White => 0,
            PixelColor::Black => 1,
            PixelColor::CapStone => 2,
            PixelColor::Alignment => 3,
            PixelColor::Tmp1 => 4,
            PixelColor::Tmp2 => 5,
            PixelColor::Discarded(x) => x + 6,
        }
    }
}

impl PartialEq<u8> for PixelColor {
    fn eq(&self, other: &u8) -> bool {
        let rep: u8 = (*self).into();
        rep == *other
    }
}

pub trait AreaFiller {
    fn update(&mut self, row: Row);
}

impl<F> AreaFiller for F where F: FnMut(Row) -> () {
    fn update(&mut self, row: Row) {
        self(row)
    }
}

struct AreaCounter(usize);

impl AreaFiller for AreaCounter {
    fn update(&mut self, row: Row) {
        self.0 += row.right - row.left + 1;
    }
}

impl SearchableImage {
    /// Given an image, create a searchable copy of it
    ///
    /// This first converts the image to greyscale before filling its own buffer
    #[cfg(feature = "img")]
    pub fn from_dynamic(img: &image::DynamicImage) -> Self {
        use image::GenericImageView;
        let gray = img.to_luma();
        let w = gray.width() as usize;
        let h = gray.height() as usize;
        SearchableImage::from_greyscale(w, h, |x, y| {
            img.get_pixel(x as u32, y as u32).data[0]
        })
    }

    /// Given a function with binary output, generate a searchable image
    ///
    /// If the given function returns `true` the matching pixel will be 'black'.
    pub fn from_bitmap<F>(w: usize, h: usize, mut fill: F) -> Self where F: FnMut(usize, usize) -> bool {
        let capacity = w.checked_mul(h).expect("Image dimensions caused overflow");
        let mut pixels = Vec::with_capacity(capacity);

        for y in 0..h {
            for x in 0..w {
                let col = if fill(x, y) {
                    PixelColor::Black
                } else {
                    PixelColor::White
                };
                pixels.push(col.into())
            }
        }
        let pixels = pixels.into_boxed_slice();

        SearchableImage {
            w,
            h,
            pixels,
            unclaimed_regions: [Region::Unclaimed {
                color: PixelColor::White,
                pixel_count: 0,
                src_x: 0,
                src_y: 0,
            }; 250],
            unclaimed_count: 0,
        }
    }

    /// Given a byte valued function, generate a searchable image
    ///
    /// The values returned by the function are interpreted as luminance. i.e. a value of
    /// 0 is black, 255 is white.
    pub fn from_greyscale<F>(w: usize,
                             h: usize,
                             mut fill: F,
    ) -> Self where F: FnMut(usize, usize) -> u8 {
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
                    1
                } else {
                    0
                };
                data.push(fill);
            }
        }

        let pixels = data.into_boxed_slice();
        SearchableImage {
            w,
            h,
            pixels,
            unclaimed_regions: [Region::Unclaimed {
                color: PixelColor::White,
                pixel_count: 0,
                src_x: 0,
                src_y: 0,
            }; 250],
            unclaimed_count: 0,
        }
    }

    /// Return the width of the image
    pub fn width(&self) -> usize {
        self.w
    }

    /// Return the height of the image
    pub fn height(&self) -> usize {
        self.h
    }

    pub(crate) fn reset_regions(&mut self)  {
        self.unclaimed_count = 0;
    }

    pub(crate) fn get_region(&mut self, (x, y): (usize, usize)) -> Region {
        let color: PixelColor = self[(x, y)].into();
        match color {
            PixelColor::Discarded(r) if r < self.unclaimed_count => self.unclaimed_regions[r as usize],
            PixelColor::Discarded(_)
            | PixelColor::Black => {
                let mut next_reg_col = PixelColor::Discarded(self.unclaimed_count);
                // check if we try to recolor with the same color
                if color == self.unclaimed_count + 6 {
                    self.unclaimed_count += 1;
                    if self.unclaimed_count == 250 {
                        self.unclaimed_count = 0;
                    }
                    next_reg_col = PixelColor::Discarded(self.unclaimed_count)
                }
                let counter = self.repaint_and_apply((x, y), next_reg_col, AreaCounter(0));
                let new_reg = Region::Unclaimed {
                    color: next_reg_col,
                    src_x: x,
                    src_y: y,
                    pixel_count: counter.0,
                };
                self.unclaimed_regions[self.unclaimed_count as usize] = new_reg;

                self.unclaimed_count += 1;
                if self.unclaimed_count == 250 {
                    self.unclaimed_count = 0;
                }
                new_reg
            }
            PixelColor::Tmp1 => Region::Tmp1,
            PixelColor::Tmp2 => Region::Tmp2,
            PixelColor::Alignment => Region::Alignment,
            PixelColor::CapStone => Region::CapStone,
            PixelColor::White => panic!("Tried to color white patch"),
        }
    }

    pub(crate) fn repaint_and_apply<F>(&mut self, (x, y): (usize, usize), target_color: PixelColor, fill: F) -> F where F: AreaFiller {
        let src = self[(x, y)];
        if PixelColor::White == src || target_color == src {
            panic!("Cannot repaint with white or same color");
        }

        self.flood_fill(x, y, src, target_color.into(), fill)
    }

    fn flood_fill<F>(
        &mut self,
        x: usize,
        y: usize,
        from: u8,
        to: u8,
        mut fill: F,
    ) -> F where F: AreaFiller {
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

            fill.update(Row {
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
        fill
    }
}


impl Index<(Range<usize>, usize)> for SearchableImage {
    type Output = [u8];

    fn index(&self, (xs, y): (Range<usize>, usize)) -> &<Self as Index<(Range<usize>, usize)>>::Output {
        let start = y * self.w + xs.start;
        let end = y * self.w + xs.end;
        &self.pixels[start..end]
    }
}

impl IndexMut<(Range<usize>, usize)> for SearchableImage {
    fn index_mut(&mut self, (xs, y): (Range<usize>, usize)) -> &mut <Self as Index<(Range<usize>, usize)>>::Output {
        let start = y * self.w + xs.start;
        let end = y * self.w + xs.end;
        &mut self.pixels[start..end]
    }
}

impl Index<Point> for SearchableImage {
    type Output = u8;

    fn index(&self, index: Point) -> &<Self as Index<Point>>::Output {
        assert!(index.x >= 0);
        assert!((index.x as usize) < self.w);
        assert!(index.y >= 0);
        assert!((index.y as usize) < self.h);

        &self[(index.x as usize, index.y as usize)]
    }
}

impl IndexMut<Point> for SearchableImage {
    fn index_mut(&mut self, index: Point) -> &mut <Self as Index<Point>>::Output {
        assert!(index.x >= 0);
        assert!((index.x as usize) < self.w);
        assert!(index.y >= 0);
        assert!((index.y as usize) < self.h);

        &mut self[(index.x as usize, index.y as usize)]
    }
}


impl Index<(usize, usize)> for SearchableImage {
    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> &<Self as Index<(usize, usize)>>::Output {
        &self.pixels[y * self.w + x]
    }
}

impl IndexMut<(usize, usize)> for SearchableImage {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut <Self as Index<(usize, usize)>>::Output {
        &mut self.pixels[y * self.w + x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img_from_array(array: [[u8; 3]; 3]) -> SearchableImage {
        let mut pixels = Vec::new();
        for col in array.iter() {
            for item in col.iter() {
                if *item == 0 {
                    pixels.push(0)
                } else {
                    pixels.push(1)
                }
            }
        }

        SearchableImage {
            w: 3,
            h: 3,
            pixels: pixels.into_boxed_slice(),
            unclaimed_count: 0,
            unclaimed_regions: [Region::Unclaimed {
                src_y: 0,
                src_x: 0,
                color: PixelColor::White,
                pixel_count: 0,
            }; 250],
        }
    }

    #[test]
    fn test_flood_fill_full() {
        let mut test_full = img_from_array([
            [1, 1, 1],
            [1, 1, 1],
            [1, 1, 1],
        ]);

        test_full.flood_fill(0, 0, 1, 2, &mut |_| ());

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(test_full[(x, y)], 2);
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

        test_single.flood_fill(1, 1, 1, 2, &mut |_| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && y == 1 {
                    assert_eq!(test_single[(x, y)], 2);
                } else {
                    let col = if (x + y) % 2 == 0 {
                        1
                    } else {
                        0
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

        test_ring.flood_fill(0, 0, 1, 2, &mut |_| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && y == 1 {
                    assert_eq!(test_ring[(x, y)], 0);
                } else {
                    assert_eq!(test_ring[(x, y)], 2);
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


        test_u.flood_fill(0, 0, 1, 2, &mut |_| ());

        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && (y == 0 || y == 1) {
                    assert_eq!(test_u[(x, y)], 0);
                } else {
                    assert_eq!(test_u[(x, y)], 2);
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

        test_empty.flood_fill(1, 1, 1, 2, &mut |_| ());

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(test_empty[(x, y)], 0)
            }
        }
    }

    #[test]
    fn test_get_region() {
        let mut test_u = img_from_array([
            [1, 0, 1],
            [1, 0, 1],
            [1, 1, 1],
        ]);

        let reg = test_u.get_region((0, 0));
        let (color, src_x, src_y, pixel_count) = match reg {
            Region::Unclaimed {
                color,
                src_x,
                src_y,
                pixel_count,
            } => (color, src_x, src_y, pixel_count),
            x @ _ => panic!("Expected Region::Unclaimed, got {:?}", x),
        };
        assert_eq!(0, src_x);
        assert_eq!(0, src_y);
        assert_eq!(7, pixel_count);
        for x in 0..3 {
            for y in 0..3 {
                if x == 1 && (y == 0 || y == 1) {
                    assert_eq!(PixelColor::White, test_u[(x, y)]);
                } else {
                    assert_eq!(color, test_u[(x, y)]);
                }
            }
        }
    }
}
