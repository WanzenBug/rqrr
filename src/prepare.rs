use std::cmp;

use crate::identify::Point;
use lru::LruCache;

/// An black-and-white image that can be mutated on search for QR codes
///
/// During search for QR codes, some black zones will be recolored in 'different' shades of black.
/// This is done to speed up the search and mitigate the impact of a huge zones.
pub struct PreparedImage<S> {
    buffer: S,
    cache: LruCache<u8, ColoredRegion>,
}

impl<S> Clone for PreparedImage<S> where S: Clone {
    fn clone(&self) -> Self {
        let mut cache = LruCache::new(self.cache.cap());
        for (k, v) in self.cache.iter() {
            cache.put(*k, v.clone());
        }

        PreparedImage {
            buffer: self.buffer.clone(),
            cache,
        }
    }
}

pub trait ImageBuffer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get_pixel(&self, x: usize, y: usize) -> u8;
    fn set_pixel(&mut self, x: usize, y: usize, val: u8);
}

#[cfg(feature = "img")]
impl ImageBuffer for image::GrayImage {
    fn width(&self) -> usize {
        self.width() as usize
    }

    fn height(&self) -> usize {
        self.height() as usize
    }

    fn get_pixel(&self, x: usize, y: usize) -> u8 {
        self.get_pixel(x as u32, y as u32).data[0]
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        self.get_pixel_mut(x as u32, y as u32).data[0] = val;
    }
}

#[derive(Clone, Debug)]
pub struct BasicImageBuffer {
    w: usize,
    h: usize,
    pixels: Box<[u8]>,
}

impl ImageBuffer for BasicImageBuffer {
    fn width(&self) -> usize {
        self.w
    }

    fn height(&self) -> usize {
        self.h
    }

    fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let w = self.width();
        self.pixels[(y * w) + x]
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        let w = self.width();
        self.pixels[(y * w) + x] = val
    }
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
    Discarded(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColoredRegion {
    Unclaimed {
        color: PixelColor,
        src_x: usize,
        src_y: usize,
        pixel_count: usize,
    },
    CapStone,
    Alignment,
    Tmp1,
}

impl From<u8> for PixelColor {
    fn from(x: u8) -> Self {
        match x {
            0 => PixelColor::White,
            1 => PixelColor::Black,
            2 => PixelColor::CapStone,
            3 => PixelColor::Alignment,
            4 => PixelColor::Tmp1,
            x => PixelColor::Discarded(x - 5),
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
            PixelColor::Discarded(x) => x + 5,
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

impl<S> PreparedImage<S> where S: ImageBuffer {
    pub fn prepare(mut buf: S) -> Self {
        let w = buf.width();
        let h = buf.height();
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
                avg_v = avg_v * (threshold_s - 1) / threshold_s + buf.get_pixel(v, y) as usize;
                avg_u = avg_u * (threshold_s - 1) / threshold_s + buf.get_pixel(u, y) as usize;
                row_average[v] += avg_v;
                row_average[u] += avg_u;
            }

            for x in 0..w {
                let fill = if (buf.get_pixel(x, y) as usize) < row_average[x] * (100 - 5) / (200 * threshold_s) {
                    PixelColor::Black
                } else {
                    PixelColor::White
                };
                buf.set_pixel(x, y, fill.into());
            }
        }

        PreparedImage {
            buffer: buf,
            cache: LruCache::new(251)
        }
    }

    pub fn without_preparation(buf: S) -> Self {
        for y in 0..buf.height() {
            for x in 0..buf.width() {
                assert!(buf.get_pixel(x, y) < 2);
            }
        }

        PreparedImage {
            buffer: buf,
            cache: LruCache::new(251)
        }
    }

    /// Return the width of the image
    pub fn width(&self) -> usize {
        self.buffer.width()
    }

    /// Return the height of the image
    pub fn height(&self) -> usize {
        self.buffer.height()
    }

    pub(crate) fn get_region(&mut self, (x, y): (usize, usize)) -> ColoredRegion {
        let color: PixelColor = self.buffer.get_pixel(x, y).into();
        match color {
            PixelColor::Discarded(r) => {
                self.cache.get(&r).unwrap().clone()
            }
            PixelColor::Black => {
                let cache_fill = self.cache.len();
                let reg_idx = if cache_fill == self.cache.cap() {
                    let (c, reg) = self.cache.pop_lru().expect("fill is at capacity (251)");
                    match reg {
                        ColoredRegion::Unclaimed {
                            src_x,
                            src_y,
                            color,
                            ..
                        } => {
                            self.flood_fill(src_x, src_y, color.into(), PixelColor::Black.into(), |_| ());
                        }
                        _ => (),
                    }
                    c
                } else {
                    cache_fill as u8
                };
                let next_reg_color = PixelColor::Discarded(reg_idx);
                let counter = self.repaint_and_apply((x, y), next_reg_color, AreaCounter(0));
                let new_reg = ColoredRegion::Unclaimed {
                    color: next_reg_color,
                    src_x: x,
                    src_y: y,
                    pixel_count: counter.0,
                };
                self.cache.put(reg_idx, new_reg);
                new_reg
            }
            PixelColor::Tmp1 => ColoredRegion::Tmp1,
            PixelColor::Alignment => ColoredRegion::Alignment,
            PixelColor::CapStone => ColoredRegion::CapStone,
            PixelColor::White => panic!("Tried to color white patch"),
        }
    }

    pub(crate) fn repaint_and_apply<F>(&mut self, (x, y): (usize, usize), target_color: PixelColor, fill: F) -> F where F: AreaFiller {
        let src = self.buffer.get_pixel(x, y);
        if PixelColor::White == src || target_color == src {
            panic!("Cannot repaint with white or same color");
        }

        self.flood_fill(x, y, src, target_color.into(), fill)
    }

    pub fn get_pixel_at_point(&self, p: Point) -> PixelColor {
        if p.x < 0 || p.x as usize >= self.width() {
            panic!("Out of bounds pixel access");
        }
        if p.y < 0 || p.y as usize >= self.height() {
            panic!("Out of bounds pixel access");
        }

        self.buffer.get_pixel(p.x as usize, p.y as usize).into()
    }

    pub fn get_pixel_at(&self, x: usize, y: usize) -> PixelColor {
        self.buffer.get_pixel(x, y).into()
    }

    #[cfg(feature = "img")]
    pub fn write_state_to(&self, p: &str) {
        let mut dyn_img = image::RgbImage::new(self.width() as u32, self.height() as u32);
        const COLORS: [[u8; 3]; 8] = [
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [255, 255, 0],
            [255, 0, 255],
            [0, 255, 255],
            [128, 128, 128],
            [128, 0, 128],
        ];
        for y in 0..self.height() {
            for x in 0..self.width() {
                let px = self.buffer.get_pixel(x, y);
                dyn_img.get_pixel_mut(x as u32, y as u32).data = if px == 0 {
                    [255, 255, 255]
                } else if px == 1 {
                    [0, 0, 0]
                } else {
                    let i = self.buffer.get_pixel(x, y) - 2;
                    COLORS[(i % 8) as usize]
                }
            }
        }
        dyn_img.save(p).unwrap();
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
        let w = self.width();
        let mut queue = Vec::new();
        queue.push((x, y));

        while let Some((x, y)) = queue.pop() {
            // Bail early in case there is nothing to fill
            if self.buffer.get_pixel(x, y) == to || self.buffer.get_pixel(x, y) != from {
                continue;
            }

            let mut left = x;
            let mut right = x;

            while left > 0 && self.buffer.get_pixel(left - 1, y) == from {
                left -= 1;
            }
            while right < w - 1 && self.buffer.get_pixel(right + 1, y) == from {
                right += 1
            }

            /* Fill the extent */
            for idx in left..=right {
                self.buffer.set_pixel(idx, y, to);
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
                    let p = self.buffer.get_pixel(x, y - 1);
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
            if y < self.height() - 1 {
                let mut seeded_previous = false;
                for x in left..=right {
                    let p = self.buffer.get_pixel(x, y + 1);
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

impl PreparedImage<BasicImageBuffer> {
    /// Given a function with binary output, generate a searchable image
    ///
    /// If the given function returns `true` the matching pixel will be 'black'.
    pub fn prepare_from_bitmap<F>(w: usize, h: usize, mut fill: F) -> Self where F: FnMut(usize, usize) -> bool {
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
        let buffer = BasicImageBuffer {
            w,
            h,
            pixels,
        };
        PreparedImage::without_preparation(buffer)
    }

    /// Given a byte valued function, generate a searchable image
    ///
    /// The values returned by the function are interpreted as luminance. i.e. a value of
    /// 0 is black, 255 is white.
    pub fn prepare_from_greyscale<F>(w: usize,
                                     h: usize,
                                     mut fill: F,
    ) -> Self where F: FnMut(usize, usize) -> u8 {
        let capacity = w.checked_mul(h).expect("Image dimensions caused overflow");
        let mut data = Vec::with_capacity(capacity);
        for y in 0..h {
            for x in 0..w {
                data.push(fill(x, y));
            }
        }
        let pixels = data.into_boxed_slice();
        let buffer = BasicImageBuffer {
            w,
            h,
            pixels,
        };
        PreparedImage::prepare(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img_from_array(array: [[u8; 3]; 3]) -> PreparedImage<BasicImageBuffer> {
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
        let buffer = BasicImageBuffer {
            w: 3,
            h: 3,
            pixels: pixels.into_boxed_slice(),
        };

        PreparedImage {
            buffer,
            cache: LruCache::new(251),
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
                assert_eq!(test_full.get_pixel_at(x, y), 2);
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
                    assert_eq!(test_single.get_pixel_at(x, y), 2);
                } else {
                    let col = if (x + y) % 2 == 0 {
                        1
                    } else {
                        0
                    };
                    assert_eq!(test_single.get_pixel_at(x, y), col);
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
                    assert_eq!(test_ring.get_pixel_at(x, y), 0);
                } else {
                    assert_eq!(test_ring.get_pixel_at(x, y), 2);
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
                    assert_eq!(test_u.get_pixel_at(x, y), 0);
                } else {
                    assert_eq!(test_u.get_pixel_at(x, y), 2);
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
                assert_eq!(test_empty.get_pixel_at(x, y), 0)
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
            ColoredRegion::Unclaimed {
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
                    assert_eq!(PixelColor::White, test_u.get_pixel_at(x, y));
                } else {
                    assert_eq!(color, test_u.get_pixel_at(x, y));
                }
            }
        }
    }
}
