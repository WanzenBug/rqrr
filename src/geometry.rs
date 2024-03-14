use crate::identify::Point;

#[derive(Debug, PartialEq, Clone)]
pub struct Perspective(pub [f64; 8]);

impl Perspective {
    pub fn create(rect: &[Point; 4], w: f64, h: f64) -> Option<Self> {
        let mut c = [0.0; 8];
        let x0 = rect[0].x as f64;
        let y0 = rect[0].y as f64;
        let x1 = rect[1].x as f64;
        let y1 = rect[1].y as f64;
        let x2 = rect[2].x as f64;
        let y2 = rect[2].y as f64;
        let x3 = rect[3].x as f64;
        let y3 = rect[3].y as f64;
        let wden = w * (x2 * y3 - x3 * y2 + (x3 - x2) * y1 + x1 * (y2 - y3));
        let hden = h * (x2 * y3 + x1 * (y2 - y3) - x3 * y2 + (x3 - x2) * y1);

        if wden < f64::EPSILON || hden < f64::EPSILON {
            return None;
        }

        c[0] = (x1 * (x2 * y3 - x3 * y2)
            + x0 * (-x2 * y3 + x3 * y2 + (x2 - x3) * y1)
            + x1 * (x3 - x2) * y0)
            / wden;
        c[1] = -(x0 * (x2 * y3 + x1 * (y2 - y3) - x2 * y1) - x1 * x3 * y2
            + x2 * x3 * y1
            + (x1 * x3 - x2 * x3) * y0)
            / hden;
        c[2] = x0;
        c[3] = (y0 * (x1 * (y3 - y2) - x2 * y3 + x3 * y2)
            + y1 * (x2 * y3 - x3 * y2)
            + x0 * y1 * (y2 - y3))
            / wden;
        c[4] = (x0 * (y1 * y3 - y2 * y3) + x1 * y2 * y3 - x2 * y1 * y3
            + y0 * (x3 * y2 - x1 * y2 + (x2 - x3) * y1))
            / hden;
        c[5] = y0;
        c[6] = (x1 * (y3 - y2) + x0 * (y2 - y3) + (x2 - x3) * y1 + (x3 - x2) * y0) / wden;
        c[7] = (-x2 * y3 + x1 * y3 + x3 * y2 + x0 * (y1 - y2) - x3 * y1 + (x2 - x1) * y0) / hden;

        Some(Perspective(c))
    }

    pub fn map(&self, u: f64, v: f64) -> Point {
        let den = self.0[6] * u + self.0[7] * v + 1.0f64;
        let x = (self.0[0] * u + self.0[1] * v + self.0[2]) / den;
        let y = (self.0[3] * u + self.0[4] * v + self.0[5]) / den;

        let x = x.round();
        let y = y.round();

        assert!(x <= i32::MAX as f64);
        assert!(x >= i32::MIN as f64);
        assert!(y <= i32::MAX as f64);
        assert!(y >= i32::MIN as f64);
        Point {
            x: x as i32,
            y: y as i32,
        }
    }

    pub fn unmap(&self, p: &Point) -> (f64, f64) {
        let x = p.x as f64;
        let y = p.y as f64;
        let den = -self.0[0] * self.0[7] * y
            + self.0[1] * self.0[6] * y
            + (self.0[3] * self.0[7] - self.0[4] * self.0[6]) * x
            + self.0[0] * self.0[4]
            - self.0[1] * self.0[3];
        let u = -(self.0[1] * (y - self.0[5]) - self.0[2] * self.0[7] * y
            + (self.0[5] * self.0[7] - self.0[4]) * x
            + self.0[2] * self.0[4])
            / den;
        let v = (self.0[0] * (y - self.0[5]) - self.0[2] * self.0[6] * y
            + (self.0[5] * self.0[6] - self.0[3]) * x
            + self.0[2] * self.0[3])
            / den;

        (u, v)
    }
}

pub fn line_intersect(p0: &Point, p1: &Point, q0: &Point, q1: &Point) -> Option<Point> {
    /* (a, b) is perpendicular to line p */
    let a = -(p1.y - p0.y);
    let b = p1.x - p0.x;
    /* (c, d) is perpendicular to line q */
    let c = -(q1.y - q0.y);
    let d = q1.x - q0.x;
    /* e and f are dot products of the respective vectors with p and q */
    let e = a * p1.x + b * p1.y;
    let f = c * q1.x + d * q1.y;
    /* Now we need to solve:
     *     [a b] [rx]   [e]
     *     [c d] [ry] = [f]
     *
     * We do this by inverting the matrix and applying it to (e, f):
     *       [ d -b] [e]   [rx]
     * 1/det [-c  a] [f] = [ry]
     */
    let det = a * d - b * c;
    if det == 0 {
        None
    } else {
        Some(Point {
            x: (d * e - b * f) / det,
            y: (-c * e + a * f) / det,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BresenhamScan {
    x: i32,
    y: i32,
    dom_step: i32,
    non_step: i32,
    i: i32,
    d: i32,
    n: i32,
    a: i32,
    x_dom: bool,
}

impl BresenhamScan {
    pub fn new(from: &Point, to: &Point) -> Self {
        let n = to.x - from.x;
        let d = to.y - from.y;
        let x = from.x;
        let y = from.y;

        let (x_dom, n, d) = if n.abs() > d.abs() {
            (true, d, n)
        } else {
            (false, n, d)
        };

        let (n, non_step) = if n < 0 { (-n, -1) } else { (n, 1) };

        let (d, dom_step) = if d < 0 { (-d, -1) } else { (d, 1) };

        BresenhamScan {
            x,
            y,
            dom_step,
            non_step,
            i: 0,
            d,
            n,
            a: n,
            x_dom,
        }
    }
}

impl Iterator for BresenhamScan {
    type Item = Point;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.i > self.d {
            return None;
        }

        let ret = Point {
            x: self.x,
            y: self.y,
        };

        self.a += self.n;
        let (dom, non) = match (self.x_dom, &mut self.x, &mut self.y) {
            (true, x, y) => (x, y),
            (false, x, y) => (y, x),
        };

        *dom += self.dom_step;
        if self.a >= self.d {
            *non += self.non_step;
            self.a -= self.d;
        }
        self.i += 1;
        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bresenham_straight() {
        let middle = Point { x: 100, y: 100 };

        let up = Point { x: 100, y: 200 };

        let right = Point { x: 300, y: 100 };

        let scan_up = BresenhamScan::new(&middle, &up);
        for (i, p) in scan_up.enumerate() {
            assert_eq!(100 + i as i32, p.y);
            assert_eq!(100, p.x);
        }

        let scan_down = BresenhamScan::new(&up, &middle);
        for (i, p) in scan_down.enumerate() {
            assert_eq!(200 - i as i32, p.y);
            assert_eq!(100, p.x);
        }

        let scan_right = BresenhamScan::new(&middle, &right);
        for (i, p) in scan_right.enumerate() {
            assert_eq!(100, p.y);
            assert_eq!(100 + i as i32, p.x);
        }

        let scan_right = BresenhamScan::new(&right, &middle);
        for (i, p) in scan_right.enumerate() {
            assert_eq!(100, p.y);
            assert_eq!(300 - i as i32, p.x);
        }
    }

    #[test]
    fn test_bresenham_zero() {
        let start = Point { x: 37, y: 45 };

        let mut scan = BresenhamScan::new(&start, &start);
        assert_eq!(scan.next(), Some(Point { x: 37, y: 45 }));
        assert_eq!(scan.next(), None)
    }

    #[test]
    fn test_bresenham_major_x() {
        // Taken from https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm

        let start = Point { x: 1, y: 1 };
        let end = Point { x: 11, y: 5 };

        let mut scan = BresenhamScan::new(&start, &end);

        assert_eq!(scan.next(), Some(Point { x: 1, y: 1 }));
        assert_eq!(scan.next(), Some(Point { x: 2, y: 1 }));
        assert_eq!(scan.next(), Some(Point { x: 3, y: 2 }));
        assert_eq!(scan.next(), Some(Point { x: 4, y: 2 }));
        assert_eq!(scan.next(), Some(Point { x: 5, y: 3 }));
        assert_eq!(scan.next(), Some(Point { x: 6, y: 3 }));
        assert_eq!(scan.next(), Some(Point { x: 7, y: 3 }));
        assert_eq!(scan.next(), Some(Point { x: 8, y: 4 }));
        assert_eq!(scan.next(), Some(Point { x: 9, y: 4 }));
        assert_eq!(scan.next(), Some(Point { x: 10, y: 5 }));
        assert_eq!(scan.next(), Some(Point { x: 11, y: 5 }));
        assert_eq!(scan.next(), None);
    }

    #[test]
    fn test_bresenham_major_y() {
        // Taken from https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm

        let start = Point { x: 5, y: 11 };
        let end = Point { x: 1, y: 1 };

        let mut scan = BresenhamScan::new(&start, &end);

        assert_eq!(scan.next(), Some(Point { x: 5, y: 11 }));
        assert_eq!(scan.next(), Some(Point { x: 5, y: 10 }));
        assert_eq!(scan.next(), Some(Point { x: 4, y: 9 }));
        assert_eq!(scan.next(), Some(Point { x: 4, y: 8 }));
        assert_eq!(scan.next(), Some(Point { x: 3, y: 7 }));
        assert_eq!(scan.next(), Some(Point { x: 3, y: 6 }));
        assert_eq!(scan.next(), Some(Point { x: 3, y: 5 }));
        assert_eq!(scan.next(), Some(Point { x: 2, y: 4 }));
        assert_eq!(scan.next(), Some(Point { x: 2, y: 3 }));
        assert_eq!(scan.next(), Some(Point { x: 1, y: 2 }));
        assert_eq!(scan.next(), Some(Point { x: 1, y: 1 }));
        assert_eq!(scan.next(), None);
    }

    #[test]
    fn test_line_intersect_parallel() {
        let p0 = Point { x: 0, y: 0 };

        let p1 = Point { x: 0, y: 10 };

        let q0 = Point { x: 1, y: 1 };

        let q1 = Point { x: 1, y: -9 };

        assert_eq!(line_intersect(&p0, &p1, &q0, &q1), None)
    }

    #[test]
    fn test_line_intersect_values() {
        let p0 = Point { x: 0, y: 0 };

        let p1 = Point { x: 0, y: 10 };

        let q0 = Point { x: 1, y: 1 };

        let q1 = Point { x: 10, y: -9 };

        // Check that all permutations produce same result
        assert_eq!(
            line_intersect(&p0, &p1, &q0, &q1),
            Some(Point { x: 0, y: 2 })
        );
        assert_eq!(
            line_intersect(&p0, &p1, &q1, &q0),
            Some(Point { x: 0, y: 2 })
        );
        assert_eq!(
            line_intersect(&p1, &p0, &q0, &q1),
            Some(Point { x: 0, y: 2 })
        );
        assert_eq!(
            line_intersect(&p1, &p0, &q1, &q0),
            Some(Point { x: 0, y: 2 })
        );
    }
}
