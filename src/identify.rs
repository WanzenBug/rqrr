
/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
#[derive(Copy, Clone)]
pub struct quirc {
    pub image: *mut u8,
    pub pixels: *mut quirc_pixel_t,
    pub row_average: *mut i32,
    pub w: i32,
    pub h: i32,
    pub num_regions: i32,
    pub regions: [quirc_region; 254],
    pub num_capstones: i32,
    pub capstones: [quirc_capstone; 32],
    pub num_grids: i32,
    pub grids: [quirc_grid; 8],
}
#[derive(Copy, Clone)]
pub struct quirc_grid {
    pub caps: [i32; 3],
    pub align_region: i32,
    pub align: quirc_point,
    pub tpep: [quirc_point; 3],
    pub hscan: i32,
    pub vscan: i32,
    pub grid_size: i32,
    pub c: [f64; 8],
}
/* This structure describes a location in the input image buffer. */
#[derive(Copy, Clone)]
pub struct quirc_point {
    pub x: i32,
    pub y: i32,
}
#[derive(Copy, Clone)]
pub struct quirc_capstone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [quirc_point; 4],
    pub center: quirc_point,
    pub c: [f64; 8],
    pub qr_grid: i32,
}
#[derive(Copy, Clone)]
pub struct quirc_region {
    pub seed: quirc_point,
    pub count: i32,
    pub capstone: i32,
}
/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
pub type quirc_pixel_t = u8;
#[derive(Copy, Clone)]
pub struct neighbour_list {
    pub n: [neighbour; 32],
    pub count: i32,
}
#[derive(Copy, Clone)]
pub struct neighbour {
    pub index: i32,
    pub distance: f64,
}
#[derive(Copy, Clone)]
pub struct quirc_version_info {
    pub data_bytes: i32,
    pub apat: [i32; 7],
    pub ecc: [quirc_rs_params; 4],
}
/* ***********************************************************************
 * QR-code version information database
 */
#[derive(Copy, Clone)]
pub struct quirc_rs_params {
    pub bs: i32,
    pub dw: i32,
    pub ns: i32,
}
#[derive(Copy, Clone)]
pub struct polygon_score_data {
    pub ref_0: quirc_point,
    pub scores: [i32; 4],
    pub corners: *mut quirc_point,
}
/* ***********************************************************************
 * Span-based floodfill routine
 */
pub type span_func_t = Option<
    unsafe extern "C" fn(_: *mut libc::c_void, _: i32, _: i32, _: i32)
        -> (),
>;
/* Limits on the maximum size of QR-codes and their content. */
/* QR-code ECC types. */
/* QR-code data types. */
/* Common character encodings */
/* This structure is used to return information about detected QR codes
 * in the input image.
 */
#[derive(Copy, Clone)]
pub struct quirc_code {
    pub corners: [quirc_point; 4],
    pub size: i32,
    pub cell_bitmap: [u8; 3917],
}
/* These functions are used to process images for QR-code recognition.
 * quirc_begin() must first be called to obtain access to a buffer into
 * which the input image should be placed. Optionally, the current
 * width and height may be returned.
 *
 * After filling the buffer, quirc_end() should be called to process
 * the image for QR-code recognition. The locations and content of each
 * code may be obtained using accessor functions described below.
 */
#[no_mangle]
pub unsafe extern "C" fn quirc_begin(
    mut q: *mut quirc,
    mut w: *mut i32,
    mut h: *mut i32,
) -> *mut u8 {
    (*q).num_regions = 2i32;
    (*q).num_capstones = 0i32;
    (*q).num_grids = 0i32;
    if !w.is_null() {
        *w = (*q).w
    }
    if !h.is_null() {
        *h = (*q).h
    }
    return (*q).image;
}
#[no_mangle]
pub unsafe extern "C" fn quirc_end(mut q: *mut quirc) -> () {
    let mut i: i32 = 0;
    pixels_setup(q);
    threshold(q);
    i = 0i32;
    while i < (*q).h {
        finder_scan(q, i);
        i += 1
    }
    i = 0i32;
    while i < (*q).num_capstones {
        test_grouping(q, i);
        i += 1
    }
}
unsafe extern "C" fn test_grouping(mut q: *mut quirc, mut i: i32) -> () {
    let mut n: *mut neighbour = 0 as *mut neighbour;
    let mut c1: *mut quirc_capstone = &mut (*q).capstones[i as usize] as *mut quirc_capstone;
    let mut j: i32 = 0;
    let mut hlist: neighbour_list = neighbour_list {
        n: [neighbour {
            index: 0,
            distance: 0.,
        }; 32],
        count: 0,
    };
    let mut vlist: neighbour_list = neighbour_list {
        n: [neighbour {
            index: 0,
            distance: 0.,
        }; 32],
        count: 0,
    };
    if (*c1).qr_grid >= 0i32 {
        return;
    } else {
        hlist.count = 0i32;
        vlist.count = 0i32;
        /* Look for potential neighbours by examining the relative gradients
         * from this capstone to others.
         */
        j = 0i32;
        while j < (*q).num_capstones {
            let mut c2: *mut quirc_capstone =
                &mut (*q).capstones[j as usize] as *mut quirc_capstone;
            let mut u: f64 = 0.;
            let mut v: f64 = 0.;
            if !(i == j || (*c2).qr_grid >= 0i32) {
                perspective_unmap((*c1).c.as_mut_ptr(), &mut (*c2).center, &mut u, &mut v);
                u = fabs(u - 3.5f64);
                v = fabs(v - 3.5f64);
                if u < 0.2f64 * v {
                    let fresh0 = hlist.count;
                    hlist.count = hlist.count + 1;
                    n = &mut hlist.n[fresh0 as usize] as *mut neighbour;
                    (*n).index = j;
                    (*n).distance = v
                }
                if v < 0.2f64 * u {
                    let fresh1 = vlist.count;
                    vlist.count = vlist.count + 1;
                    let mut n_0: *mut neighbour = &mut vlist.n[fresh1 as usize] as *mut neighbour;
                    (*n_0).index = j;
                    (*n_0).distance = u
                }
            }
            j += 1
        }
        if !(0 != hlist.count && 0 != vlist.count) {
            return;
        } else {
            test_neighbours(q, i, &mut hlist, &mut vlist);
            return;
        }
    };
}
unsafe extern "C" fn test_neighbours(
    mut q: *mut quirc,
    mut i: i32,
    mut hlist: *const neighbour_list,
    mut vlist: *const neighbour_list,
) -> () {
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut best_score: f64 = 0.0f64;
    let mut best_h: i32 = -1i32;
    let mut best_v: i32 = -1i32;
    /* Test each possible grouping */
    j = 0i32;
    while j < (*hlist).count {
        k = 0i32;
        while k < (*vlist).count {
            let mut hn: *const neighbour = &(*hlist).n[j as usize] as *const neighbour;
            let mut vn: *const neighbour = &(*vlist).n[k as usize] as *const neighbour;
            let mut score: f64 = fabs(1.0f64 - (*hn).distance / (*vn).distance);
            if !(score > 2.5f64) {
                if best_h < 0i32 || score < best_score {
                    best_h = (*hn).index;
                    best_v = (*vn).index;
                    best_score = score
                }
            }
            k += 1
        }
        j += 1
    }
    if best_h < 0i32 || best_v < 0i32 {
        return;
    } else {
        record_qr_grid(q, best_h, i, best_v);
        return;
    };
}
unsafe extern "C" fn record_qr_grid(
    mut q: *mut quirc,
    mut a: i32,
    mut b: i32,
    mut c: i32,
) -> () {
    let mut swap: i32 = 0;
    let mut h0: quirc_point = quirc_point { x: 0, y: 0 };
    let mut hd: quirc_point = quirc_point { x: 0, y: 0 };
    let mut i: i32 = 0;
    let mut qr_index: i32 = 0;
    let mut qr: *mut quirc_grid = 0 as *mut quirc_grid;
    if (*q).num_grids >= 8i32 {
        return;
    } else {
        /* Construct the hypotenuse line from A to C. B should be to
         * the left of this line.
         */
        memcpy(
            &mut h0 as *mut quirc_point as *mut libc::c_void,
            &mut (*q).capstones[a as usize].center as *mut quirc_point as *const libc::c_void,
            ::std::mem::size_of::<quirc_point>() as u64,
        );
        hd.x = (*q).capstones[c as usize].center.x - (*q).capstones[a as usize].center.x;
        hd.y = (*q).capstones[c as usize].center.y - (*q).capstones[a as usize].center.y;
        /* Make sure A-B-C is clockwise */
        if ((*q).capstones[b as usize].center.x - h0.x) * -hd.y
            + ((*q).capstones[b as usize].center.y - h0.y) * hd.x > 0i32
        {
            swap = a;
            a = c;
            c = swap;
            hd.x = -hd.x;
            hd.y = -hd.y
        }
        /* Record the grid and its components */
        qr_index = (*q).num_grids;
        let fresh2 = (*q).num_grids;
        (*q).num_grids = (*q).num_grids + 1;
        qr = &mut (*q).grids[fresh2 as usize] as *mut quirc_grid;
        memset(
            qr as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<quirc_grid>() as u64,
        );
        (*qr).caps[0usize] = a;
        (*qr).caps[1usize] = b;
        (*qr).caps[2usize] = c;
        (*qr).align_region = -1i32;
        /* Rotate each capstone so that corner 0 is top-left with respect
         * to the grid.
         */
        i = 0i32;
        while i < 3i32 {
            let mut cap: *mut quirc_capstone =
                &mut (*q).capstones[(*qr).caps[i as usize] as usize] as *mut quirc_capstone;
            rotate_capstone(cap, &mut h0, &mut hd);
            (*cap).qr_grid = qr_index;
            i += 1
        }
        /* Check the timing pattern. This doesn't require a perspective
         * transform.
         */
        if !(measure_timing_pattern(q, qr_index) < 0i32) {
            /* Make an estimate based for the alignment pattern based on extending
             * lines from capstones A and C.
             */
            if !(0 == line_intersect(
                &mut (*q).capstones[a as usize].corners[0usize],
                &mut (*q).capstones[a as usize].corners[1usize],
                &mut (*q).capstones[c as usize].corners[0usize],
                &mut (*q).capstones[c as usize].corners[3usize],
                &mut (*qr).align,
            )) {
                /* On V2+ grids, we should use the alignment pattern. */
                if (*qr).grid_size > 21i32 {
                    /* Try to find the actual location of the alignment pattern. */
                    find_alignment_pattern(q, qr_index);
                    /* Find the point of the alignment pattern closest to the
                     * top-left of the QR grid.
                     */
                    if (*qr).align_region >= 0i32 {
                        let mut psd: polygon_score_data = polygon_score_data {
                            ref_0: quirc_point { x: 0, y: 0 },
                            scores: [0; 4],
                            corners: 0 as *mut quirc_point,
                        };
                        let mut reg: *mut quirc_region =
                            &mut (*q).regions[(*qr).align_region as usize] as *mut quirc_region;
                        /* Start from some point inside the alignment pattern */
                        memcpy(
                            &mut (*qr).align as *mut quirc_point as *mut libc::c_void,
                            &mut (*reg).seed as *mut quirc_point as *const libc::c_void,
                            ::std::mem::size_of::<quirc_point>() as u64,
                        );
                        memcpy(
                            &mut psd.ref_0 as *mut quirc_point as *mut libc::c_void,
                            &mut hd as *mut quirc_point as *const libc::c_void,
                            ::std::mem::size_of::<quirc_point>() as u64,
                        );
                        psd.corners = &mut (*qr).align;
                        psd.scores[0usize] = -hd.y * (*qr).align.x + hd.x * (*qr).align.y;
                        flood_fill_seed(
                            q,
                            (*reg).seed.x,
                            (*reg).seed.y,
                            (*qr).align_region,
                            1i32,
                            None,
                            0 as *mut libc::c_void,
                            0i32,
                        );
                        flood_fill_seed(
                            q,
                            (*reg).seed.x,
                            (*reg).seed.y,
                            1i32,
                            (*qr).align_region,
                            Some(find_leftmost_to_line),
                            &mut psd as *mut polygon_score_data as *mut libc::c_void,
                            0i32,
                        );
                    }
                }
                setup_qr_perspective(q, qr_index);
                return;
            }
        }
        /* We've been unable to complete setup for this grid. Undo what we've
         * recorded and pretend it never happened.
         */
        i = 0i32;
        while i < 3i32 {
            (*q).capstones[(*qr).caps[i as usize] as usize].qr_grid = -1i32;
            i += 1
        }
        (*q).num_grids -= 1;
        return;
    };
}
/* Once the capstones are in place and an alignment point has been
 * chosen, we call this function to set up a grid-reading perspective
 * transform.
 */
unsafe extern "C" fn setup_qr_perspective(mut q: *mut quirc, mut index: i32) -> () {
    let mut qr: *mut quirc_grid = &mut (*q).grids[index as usize] as *mut quirc_grid;
    let mut rect: [quirc_point; 4] = [quirc_point { x: 0, y: 0 }; 4];
    /* Set up the perspective map for reading the grid */
    memcpy(
        &mut rect[0usize] as *mut quirc_point as *mut libc::c_void,
        &mut (*q).capstones[(*qr).caps[1usize] as usize].corners[0usize] as *mut quirc_point
            as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    memcpy(
        &mut rect[1usize] as *mut quirc_point as *mut libc::c_void,
        &mut (*q).capstones[(*qr).caps[2usize] as usize].corners[0usize] as *mut quirc_point
            as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    memcpy(
        &mut rect[2usize] as *mut quirc_point as *mut libc::c_void,
        &mut (*qr).align as *mut quirc_point as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    memcpy(
        &mut rect[3usize] as *mut quirc_point as *mut libc::c_void,
        &mut (*q).capstones[(*qr).caps[0usize] as usize].corners[0usize] as *mut quirc_point
            as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    perspective_setup(
        (*qr).c.as_mut_ptr(),
        rect.as_mut_ptr(),
        ((*qr).grid_size - 7i32) as f64,
        ((*qr).grid_size - 7i32) as f64,
    );
    jiggle_perspective(q, index);
}
unsafe extern "C" fn jiggle_perspective(mut q: *mut quirc, mut index: i32) -> () {
    let mut qr: *mut quirc_grid = &mut (*q).grids[index as usize] as *mut quirc_grid;
    let mut best: i32 = fitness_all(q, index);
    let mut pass: i32 = 0;
    let mut adjustments: [f64; 8] = [0.; 8];
    let mut i: i32 = 0;
    i = 0i32;
    while i < 8i32 {
        adjustments[i as usize] = (*qr).c[i as usize] * 0.02f64;
        i += 1
    }
    pass = 0i32;
    while pass < 5i32 {
        i = 0i32;
        while i < 16i32 {
            let mut j: i32 = i >> 1i32;
            let mut test: i32 = 0;
            let mut old: f64 = (*qr).c[j as usize];
            let mut step: f64 = adjustments[j as usize];
            let mut new: f64 = 0.;
            if 0 != i & 1i32 {
                new = old + step
            } else {
                new = old - step
            }
            (*qr).c[j as usize] = new;
            test = fitness_all(q, index);
            if test > best {
                best = test
            } else {
                (*qr).c[j as usize] = old
            }
            i += 1
        }
        i = 0i32;
        while i < 8i32 {
            adjustments[i as usize] *= 0.5f64;
            i += 1
        }
        pass += 1
    }
}
/* Compute a fitness score for the currently configured perspective
 * transform, using the features we expect to find by scanning the
 * grid.
 */
unsafe extern "C" fn fitness_all(mut q: *const quirc, mut index: i32) -> i32 {
    let mut qr: *const quirc_grid = &(*q).grids[index as usize] as *const quirc_grid;
    let mut version: i32 = ((*qr).grid_size - 17i32) / 4i32;
    let mut info: *const quirc_version_info =
        &quirc_version_db[version as usize] as *const quirc_version_info;
    let mut score: i32 = 0i32;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut ap_count: i32 = 0;
    /* Check the timing pattern */
    i = 0i32;
    while i < (*qr).grid_size - 14i32 {
        let mut expect: i32 = if 0 != i & 1i32 { 1i32 } else { -1i32 };
        score += fitness_cell(q, index, i + 7i32, 6i32) * expect;
        score += fitness_cell(q, index, 6i32, i + 7i32) * expect;
        i += 1
    }
    /* Check capstones */
    score += fitness_capstone(q, index, 0i32, 0i32);
    score += fitness_capstone(q, index, (*qr).grid_size - 7i32, 0i32);
    score += fitness_capstone(q, index, 0i32, (*qr).grid_size - 7i32);
    if version < 0i32 || version > 40i32 {
        return score;
    } else {
        /* Check alignment patterns */
        ap_count = 0i32;
        while ap_count < 7i32 && 0 != (*info).apat[ap_count as usize] {
            ap_count += 1
        }
        i = 1i32;
        while i + 1i32 < ap_count {
            score += fitness_apat(q, index, 6i32, (*info).apat[i as usize]);
            score += fitness_apat(q, index, (*info).apat[i as usize], 6i32);
            i += 1
        }
        i = 1i32;
        while i < ap_count {
            j = 1i32;
            while j < ap_count {
                score += fitness_apat(q, index, (*info).apat[i as usize], (*info).apat[j as usize]);
                j += 1
            }
            i += 1
        }
        return score;
    };
}
unsafe extern "C" fn fitness_apat(
    mut q: *const quirc,
    mut index: i32,
    mut cx: i32,
    mut cy: i32,
) -> i32 {
    return fitness_cell(q, index, cx, cy) - fitness_ring(q, index, cx, cy, 1i32)
        + fitness_ring(q, index, cx, cy, 2i32);
}
unsafe extern "C" fn fitness_ring(
    mut q: *const quirc,
    mut index: i32,
    mut cx: i32,
    mut cy: i32,
    mut radius: i32,
) -> i32 {
    let mut i: i32 = 0;
    let mut score: i32 = 0i32;
    i = 0i32;
    while i < radius * 2i32 {
        score += fitness_cell(q, index, cx - radius + i, cy - radius);
        score += fitness_cell(q, index, cx - radius, cy + radius - i);
        score += fitness_cell(q, index, cx + radius, cy - radius + i);
        score += fitness_cell(q, index, cx + radius - i, cy + radius);
        i += 1
    }
    return score;
}
unsafe extern "C" fn fitness_cell(
    mut q: *const quirc,
    mut index: i32,
    mut x: i32,
    mut y: i32,
) -> i32 {
    let mut qr: *const quirc_grid = &(*q).grids[index as usize] as *const quirc_grid;
    let mut score: i32 = 0i32;
    let mut u: i32 = 0;
    let mut v: i32 = 0;
    v = 0i32;
    while v < 3i32 {
        u = 0i32;
        while u < 3i32 {
            static mut offsets: [f64; 3] = unsafe { [0.3f64, 0.5f64, 0.7f64] };
            let mut p: quirc_point = quirc_point { x: 0, y: 0 };
            perspective_map(
                (*qr).c.as_ptr(),
                x as f64 + offsets[u as usize],
                y as f64 + offsets[v as usize],
                &mut p,
            );
            if !(p.y < 0i32 || p.y >= (*q).h || p.x < 0i32 || p.x >= (*q).w) {
                if 0 != *(*q).pixels.offset((p.y * (*q).w + p.x) as isize) {
                    score += 1
                } else {
                    score -= 1
                }
            }
            u += 1
        }
        v += 1
    }
    return score;
}
unsafe extern "C" fn perspective_map(
    mut c: *const f64,
    mut u: f64,
    mut v: f64,
    mut ret: *mut quirc_point,
) -> () {
    let mut den: f64 = *c.offset(6isize) * u + *c.offset(7isize) * v + 1.0f64;
    let mut x: f64 =
        (*c.offset(0isize) * u + *c.offset(1isize) * v + *c.offset(2isize)) / den;
    let mut y: f64 =
        (*c.offset(3isize) * u + *c.offset(4isize) * v + *c.offset(5isize)) / den;
    (*ret).x = rint(x) as i32;
    (*ret).y = rint(y) as i32;
}
unsafe extern "C" fn fitness_capstone(
    mut q: *const quirc,
    mut index: i32,
    mut x: i32,
    mut y: i32,
) -> i32 {
    x += 3i32;
    y += 3i32;
    return fitness_cell(q, index, x, y) + fitness_ring(q, index, x, y, 1i32)
        - fitness_ring(q, index, x, y, 2i32) + fitness_ring(q, index, x, y, 3i32);
}
unsafe extern "C" fn perspective_setup(
    mut c: *mut f64,
    mut rect: *const quirc_point,
    mut w: f64,
    mut h: f64,
) -> () {
    let mut x0: f64 = (*rect.offset(0isize)).x as f64;
    let mut y0: f64 = (*rect.offset(0isize)).y as f64;
    let mut x1: f64 = (*rect.offset(1isize)).x as f64;
    let mut y1: f64 = (*rect.offset(1isize)).y as f64;
    let mut x2: f64 = (*rect.offset(2isize)).x as f64;
    let mut y2: f64 = (*rect.offset(2isize)).y as f64;
    let mut x3: f64 = (*rect.offset(3isize)).x as f64;
    let mut y3: f64 = (*rect.offset(3isize)).y as f64;
    let mut wden: f64 = w * (x2 * y3 - x3 * y2 + (x3 - x2) * y1 + x1 * (y2 - y3));
    let mut hden: f64 = h * (x2 * y3 + x1 * (y2 - y3) - x3 * y2 + (x3 - x2) * y1);
    *c.offset(0isize) = (x1 * (x2 * y3 - x3 * y2)
        + x0 * (-x2 * y3 + x3 * y2 + (x2 - x3) * y1)
        + x1 * (x3 - x2) * y0) / wden;
    *c.offset(1isize) = -(x0 * (x2 * y3 + x1 * (y2 - y3) - x2 * y1) - x1 * x3 * y2
        + x2 * x3 * y1
        + (x1 * x3 - x2 * x3) * y0) / hden;
    *c.offset(2isize) = x0;
    *c.offset(3isize) = (y0 * (x1 * (y3 - y2) - x2 * y3 + x3 * y2)
        + y1 * (x2 * y3 - x3 * y2)
        + x0 * y1 * (y2 - y3)) / wden;
    *c.offset(4isize) = (x0 * (y1 * y3 - y2 * y3) + x1 * y2 * y3 - x2 * y1 * y3
        + y0 * (x3 * y2 - x1 * y2 + (x2 - x3) * y1)) / hden;
    *c.offset(5isize) = y0;
    *c.offset(6isize) = (x1 * (y3 - y2) + x0 * (y2 - y3) + (x2 - x3) * y1 + (x3 - x2) * y0) / wden;
    *c.offset(7isize) =
        (-x2 * y3 + x1 * y3 + x3 * y2 + x0 * (y1 - y2) - x3 * y1 + (x2 - x1) * y0) / hden;
}
unsafe extern "C" fn find_leftmost_to_line(
    mut user_data: *mut libc::c_void,
    mut y: i32,
    mut left: i32,
    mut right: i32,
) -> () {
    let mut psd: *mut polygon_score_data = user_data as *mut polygon_score_data;
    let mut xs: [i32; 2] = [left, right];
    let mut i: i32 = 0;
    i = 0i32;
    while i < 2i32 {
        let mut d: i32 = -(*psd).ref_0.y * xs[i as usize] + (*psd).ref_0.x * y;
        if d < (*psd).scores[0usize] {
            (*psd).scores[0usize] = d;
            (*(*psd).corners.offset(0isize)).x = xs[i as usize];
            (*(*psd).corners.offset(0isize)).y = y
        }
        i += 1
    }
}
unsafe extern "C" fn flood_fill_seed(
    mut q: *mut quirc,
    mut x: i32,
    mut y: i32,
    mut from: i32,
    mut to: i32,
    mut func: span_func_t,
    mut user_data: *mut libc::c_void,
    mut depth: i32,
) -> () {
    let mut left: i32 = x;
    let mut right: i32 = x;
    let mut i: i32 = 0;
    let mut row: *mut quirc_pixel_t = (*q).pixels.offset((y * (*q).w) as isize);
    if depth >= 4096i32 {
        return;
    } else {
        while left > 0i32 && *row.offset((left - 1i32) as isize) as i32 == from {
            left -= 1
        }
        while right < (*q).w - 1i32 && *row.offset((right + 1i32) as isize) as i32 == from {
            right += 1
        }
        /* Fill the extent */
        i = left;
        while i <= right {
            *row.offset(i as isize) = to as quirc_pixel_t;
            i += 1
        }
        if func.is_some() {
            func.expect("non-null function pointer")(user_data, y, left, right);
        }
        /* Seed new flood-fills */
        if y > 0i32 {
            row = (*q).pixels.offset(((y - 1i32) * (*q).w) as isize);
            i = left;
            while i <= right {
                if *row.offset(i as isize) as i32 == from {
                    flood_fill_seed(q, i, y - 1i32, from, to, func, user_data, depth + 1i32);
                }
                i += 1
            }
        }
        if y < (*q).h - 1i32 {
            row = (*q).pixels.offset(((y + 1i32) * (*q).w) as isize);
            i = left;
            while i <= right {
                if *row.offset(i as isize) as i32 == from {
                    flood_fill_seed(q, i, y + 1i32, from, to, func, user_data, depth + 1i32);
                }
                i += 1
            }
        }
        return;
    };
}
unsafe extern "C" fn find_alignment_pattern(mut q: *mut quirc, mut index: i32) -> () {
    let mut qr: *mut quirc_grid = &mut (*q).grids[index as usize] as *mut quirc_grid;
    let mut c0: *mut quirc_capstone =
        &mut (*q).capstones[(*qr).caps[0usize] as usize] as *mut quirc_capstone;
    let mut c2: *mut quirc_capstone =
        &mut (*q).capstones[(*qr).caps[2usize] as usize] as *mut quirc_capstone;
    let mut a: quirc_point = quirc_point { x: 0, y: 0 };
    let mut b: quirc_point = quirc_point { x: 0, y: 0 };
    let mut c: quirc_point = quirc_point { x: 0, y: 0 };
    let mut size_estimate: i32 = 0;
    let mut step_size: i32 = 1i32;
    let mut dir: i32 = 0i32;
    let mut u: f64 = 0.;
    let mut v: f64 = 0.;
    /* Grab our previous estimate of the alignment pattern corner */
    memcpy(
        &mut b as *mut quirc_point as *mut libc::c_void,
        &mut (*qr).align as *mut quirc_point as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    /* Guess another two corners of the alignment pattern so that we
     * can estimate its size.
     */
    perspective_unmap((*c0).c.as_mut_ptr(), &mut b, &mut u, &mut v);
    perspective_map((*c0).c.as_mut_ptr(), u, v + 1.0f64, &mut a);
    perspective_unmap((*c2).c.as_mut_ptr(), &mut b, &mut u, &mut v);
    perspective_map((*c2).c.as_mut_ptr(), u + 1.0f64, v, &mut c);
    size_estimate = abs((a.x - b.x) * -(c.y - b.y) + (a.y - b.y) * (c.x - b.x));
    /* Spiral outwards from the estimate point until we find something
     * roughly the right size. Don't look too far from the estimate
     * point.
     */
    while step_size * step_size < size_estimate * 100i32 {
        static mut dx_map: [i32; 4] = unsafe { [1i32, 0i32, -1i32, 0i32] };
        static mut dy_map: [i32; 4] = unsafe { [0i32, -1i32, 0i32, 1i32] };
        let mut i: i32 = 0;
        i = 0i32;
        while i < step_size {
            let mut code: i32 = region_code(q, b.x, b.y);
            if code >= 0i32 {
                let mut reg: *mut quirc_region =
                    &mut (*q).regions[code as usize] as *mut quirc_region;
                if (*reg).count >= size_estimate / 2i32 && (*reg).count <= size_estimate * 2i32 {
                    (*qr).align_region = code;
                    return;
                }
            }
            b.x += dx_map[dir as usize];
            b.y += dy_map[dir as usize];
            i += 1
        }
        dir = (dir + 1i32) % 4i32;
        if !(0 == dir & 1i32) {
            continue;
        }
        step_size += 1
    }
}
unsafe extern "C" fn region_code(
    mut q: *mut quirc,
    mut x: i32,
    mut y: i32,
) -> i32 {
    let mut pixel: i32 = 0;
    let mut box_0: *mut quirc_region = 0 as *mut quirc_region;
    let mut region: i32 = 0;
    if x < 0i32 || y < 0i32 || x >= (*q).w || y >= (*q).h {
        return -1i32;
    } else {
        pixel = *(*q).pixels.offset((y * (*q).w + x) as isize) as i32;
        if pixel >= 2i32 {
            return pixel;
        } else if pixel == 0i32 {
            return -1i32;
        } else if (*q).num_regions >= 254i32 {
            return -1i32;
        } else {
            region = (*q).num_regions;
            let fresh3 = (*q).num_regions;
            (*q).num_regions = (*q).num_regions + 1;
            box_0 = &mut (*q).regions[fresh3 as usize] as *mut quirc_region;
            memset(
                box_0 as *mut libc::c_void,
                0i32,
                ::std::mem::size_of::<quirc_region>() as u64,
            );
            (*box_0).seed.x = x;
            (*box_0).seed.y = y;
            (*box_0).capstone = -1i32;
            flood_fill_seed(
                q,
                x,
                y,
                pixel,
                region,
                Some(area_count),
                box_0 as *mut libc::c_void,
                0i32,
            );
            return region;
        }
    };
}
unsafe extern "C" fn area_count(
    mut user_data: *mut libc::c_void,
    mut y: i32,
    mut left: i32,
    mut right: i32,
) -> () {
    (*(user_data as *mut quirc_region)).count += right - left + 1i32;
}
unsafe extern "C" fn perspective_unmap(
    mut c: *const f64,
    mut in_0: *const quirc_point,
    mut u: *mut f64,
    mut v: *mut f64,
) -> () {
    let mut x: f64 = (*in_0).x as f64;
    let mut y: f64 = (*in_0).y as f64;
    let mut den: f64 = -*c.offset(0isize) * *c.offset(7isize) * y
        + *c.offset(1isize) * *c.offset(6isize) * y
        + (*c.offset(3isize) * *c.offset(7isize) - *c.offset(4isize) * *c.offset(6isize)) * x
        + *c.offset(0isize) * *c.offset(4isize)
        - *c.offset(1isize) * *c.offset(3isize);
    *u = -(*c.offset(1isize) * (y - *c.offset(5isize)) - *c.offset(2isize) * *c.offset(7isize) * y
        + (*c.offset(5isize) * *c.offset(7isize) - *c.offset(4isize)) * x
        + *c.offset(2isize) * *c.offset(4isize)) / den;
    *v = (*c.offset(0isize) * (y - *c.offset(5isize)) - *c.offset(2isize) * *c.offset(6isize) * y
        + (*c.offset(5isize) * *c.offset(6isize) - *c.offset(3isize)) * x
        + *c.offset(2isize) * *c.offset(3isize)) / den;
}
/* quirc - QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */
/* ***********************************************************************
 * Linear algebra routines
 */
unsafe extern "C" fn line_intersect(
    mut p0: *const quirc_point,
    mut p1: *const quirc_point,
    mut q0: *const quirc_point,
    mut q1: *const quirc_point,
    mut r: *mut quirc_point,
) -> i32 {
    /* (a, b) is perpendicular to line p */
    let mut a: i32 = -((*p1).y - (*p0).y);
    let mut b: i32 = (*p1).x - (*p0).x;
    /* (c, d) is perpendicular to line q */
    let mut c: i32 = -((*q1).y - (*q0).y);
    let mut d: i32 = (*q1).x - (*q0).x;
    /* e and f are dot products of the respective vectors with p and q */
    let mut e: i32 = a * (*p1).x + b * (*p1).y;
    let mut f: i32 = c * (*q1).x + d * (*q1).y;
    /* Now we need to solve:
     *     [a b] [rx]   [e]
     *     [c d] [ry] = [f]
     *
     * We do this by inverting the matrix and applying it to (e, f):
     *       [ d -b] [e]   [rx]
     * 1/det [-c  a] [f] = [ry]
     */
    let mut det: i32 = a * d - b * c;
    if 0 == det {
        return 0i32;
    } else {
        (*r).x = (d * e - b * f) / det;
        (*r).y = (-c * e + a * f) / det;
        return 1i32;
    };
}
/* Try the measure the timing pattern for a given QR code. This does
 * not require the global perspective to have been set up, but it
 * does require that the capstone corners have been set to their
 * canonical rotation.
 *
 * For each capstone, we find a point in the middle of the ring band
 * which is nearest the centre of the code. Using these points, we do
 * a horizontal and a vertical timing scan.
 */
unsafe extern "C" fn measure_timing_pattern(
    mut q: *mut quirc,
    mut index: i32,
) -> i32 {
    let mut qr: *mut quirc_grid = &mut (*q).grids[index as usize] as *mut quirc_grid;
    let mut i: i32 = 0;
    let mut scan: i32 = 0;
    let mut ver: i32 = 0;
    let mut size: i32 = 0;
    i = 0i32;
    while i < 3i32 {
        static mut us: [f64; 3] = unsafe { [6.5f64, 6.5f64, 0.5f64] };
        static mut vs: [f64; 3] = unsafe { [0.5f64, 6.5f64, 6.5f64] };
        let mut cap: *mut quirc_capstone =
            &mut (*q).capstones[(*qr).caps[i as usize] as usize] as *mut quirc_capstone;
        perspective_map(
            (*cap).c.as_mut_ptr(),
            us[i as usize],
            vs[i as usize],
            &mut (*qr).tpep[i as usize],
        );
        i += 1
    }
    (*qr).hscan = timing_scan(q, &mut (*qr).tpep[1usize], &mut (*qr).tpep[2usize]);
    (*qr).vscan = timing_scan(q, &mut (*qr).tpep[1usize], &mut (*qr).tpep[0usize]);
    scan = (*qr).hscan;
    if (*qr).vscan > scan {
        scan = (*qr).vscan
    }
    /* If neither scan worked, we can't go any further. */
    if scan < 0i32 {
        return -1i32;
    } else {
        /* Choose the nearest allowable grid size */
        size = scan * 2i32 + 13i32;
        ver = (size - 15i32) / 4i32;
        (*qr).grid_size = ver * 4i32 + 17i32;
        return 0i32;
    };
}
/* Do a Bresenham scan from one point to another and count the number
 * of black/white transitions.
 */
unsafe extern "C" fn timing_scan(
    mut q: *const quirc,
    mut p0: *const quirc_point,
    mut p1: *const quirc_point,
) -> i32 {
    let mut swap: i32 = 0;
    let mut n: i32 = (*p1).x - (*p0).x;
    let mut d: i32 = (*p1).y - (*p0).y;
    let mut x: i32 = (*p0).x;
    let mut y: i32 = (*p0).y;
    let mut dom: *mut i32 = 0 as *mut i32;
    let mut nondom: *mut i32 = 0 as *mut i32;
    let mut dom_step: i32 = 0;
    let mut nondom_step: i32 = 0;
    let mut a: i32 = 0i32;
    let mut i: i32 = 0;
    let mut run_length: i32 = 0i32;
    let mut count: i32 = 0i32;
    if (*p0).x < 0i32 || (*p0).y < 0i32 || (*p0).x >= (*q).w || (*p0).y >= (*q).h {
        return -1i32;
    } else if (*p1).x < 0i32 || (*p1).y < 0i32 || (*p1).x >= (*q).w || (*p1).y >= (*q).h {
        return -1i32;
    } else {
        if abs(n) > abs(d) {
            swap = n;
            n = d;
            d = swap;
            dom = &mut x;
            nondom = &mut y
        } else {
            dom = &mut y;
            nondom = &mut x
        }
        if n < 0i32 {
            n = -n;
            nondom_step = -1i32
        } else {
            nondom_step = 1i32
        }
        if d < 0i32 {
            d = -d;
            dom_step = -1i32
        } else {
            dom_step = 1i32
        }
        x = (*p0).x;
        y = (*p0).y;
        i = 0i32;
        while i <= d {
            let mut pixel: i32 = 0;
            if y < 0i32 || y >= (*q).h || x < 0i32 || x >= (*q).w {
                break;
            }
            pixel = *(*q).pixels.offset((y * (*q).w + x) as isize) as i32;
            if 0 != pixel {
                if run_length >= 2i32 {
                    count += 1
                }
                run_length = 0i32
            } else {
                run_length += 1
            }
            a += n;
            *dom += dom_step;
            if a >= d {
                *nondom += nondom_step;
                a -= d
            }
            i += 1
        }
        return count;
    };
}
/* Rotate the capstone with so that corner 0 is the leftmost with respect
 * to the given reference line.
 */
unsafe extern "C" fn rotate_capstone(
    mut cap: *mut quirc_capstone,
    mut h0: *const quirc_point,
    mut hd: *const quirc_point,
) -> () {
    let mut copy: [quirc_point; 4] = [quirc_point { x: 0, y: 0 }; 4];
    let mut j: i32 = 0;
    let mut best: i32 = 0;
    let mut best_score: i32 = 0;
    j = 0i32;
    while j < 4i32 {
        let mut p: *mut quirc_point = &mut (*cap).corners[j as usize] as *mut quirc_point;
        let mut score: i32 = ((*p).x - (*h0).x) * -(*hd).y + ((*p).y - (*h0).y) * (*hd).x;
        if 0 == j || score < best_score {
            best = j;
            best_score = score
        }
        j += 1
    }
    /* Rotate the capstone */
    j = 0i32;
    while j < 4i32 {
        memcpy(
            &mut copy[j as usize] as *mut quirc_point as *mut libc::c_void,
            &mut (*cap).corners[((j + best) % 4i32) as usize] as *mut quirc_point
                as *const libc::c_void,
            ::std::mem::size_of::<quirc_point>() as u64,
        );
        j += 1
    }
    memcpy(
        (*cap).corners.as_mut_ptr() as *mut libc::c_void,
        copy.as_mut_ptr() as *const libc::c_void,
        ::std::mem::size_of::<[quirc_point; 4]>() as u64,
    );
    perspective_setup(
        (*cap).c.as_mut_ptr(),
        (*cap).corners.as_mut_ptr(),
        7.0f64,
        7.0f64,
    );
}
unsafe extern "C" fn finder_scan(mut q: *mut quirc, mut y: i32) -> () {
    let mut row: *mut quirc_pixel_t = (*q).pixels.offset((y * (*q).w) as isize);
    let mut x: i32 = 0;
    let mut last_color: i32 = 0i32;
    let mut run_length: i32 = 0i32;
    let mut run_count: i32 = 0i32;
    let mut pb: [i32; 5] = [0; 5];
    memset(
        pb.as_mut_ptr() as *mut libc::c_void,
        0i32,
        ::std::mem::size_of::<[i32; 5]>() as u64,
    );
    x = 0i32;
    while x < (*q).w {
        let mut color: i32 = if 0 != *row.offset(x as isize) as i32 {
            1i32
        } else {
            0i32
        };
        if 0 != x && color != last_color {
            memmove(
                pb.as_mut_ptr() as *mut libc::c_void,
                pb.as_mut_ptr().offset(1isize) as *const libc::c_void,
                (::std::mem::size_of::<i32>() as u64)
                    .wrapping_mul(4i32 as u64),
            );
            pb[4usize] = run_length;
            run_length = 0i32;
            run_count += 1;
            if 0 == color && run_count >= 5i32 {
                static mut check: [i32; 5] = unsafe { [1i32, 1i32, 3i32, 1i32, 1i32] };
                let mut avg: i32 = 0;
                let mut err: i32 = 0;
                let mut i: i32 = 0;
                let mut ok: i32 = 1i32;
                avg = (pb[0usize] + pb[1usize] + pb[3usize] + pb[4usize]) / 4i32;
                err = avg * 3i32 / 4i32;
                i = 0i32;
                while i < 5i32 {
                    if pb[i as usize] < check[i as usize] * avg - err
                        || pb[i as usize] > check[i as usize] * avg + err
                    {
                        ok = 0i32
                    }
                    i += 1
                }
                if 0 != ok {
                    test_capstone(q, x, y, pb.as_mut_ptr());
                }
            }
        }
        run_length += 1;
        last_color = color;
        x += 1
    }
}
unsafe extern "C" fn test_capstone(
    mut q: *mut quirc,
    mut x: i32,
    mut y: i32,
    mut pb: *mut i32,
) -> () {
    let mut ring_right: i32 = region_code(q, x - *pb.offset(4isize), y);
    let mut stone: i32 = region_code(
        q,
        x - *pb.offset(4isize) - *pb.offset(3isize) - *pb.offset(2isize),
        y,
    );
    let mut ring_left: i32 = region_code(
        q,
        x
            - *pb.offset(4isize)
            - *pb.offset(3isize)
            - *pb.offset(2isize)
            - *pb.offset(1isize)
            - *pb.offset(0isize),
        y,
    );
    let mut stone_reg: *mut quirc_region = 0 as *mut quirc_region;
    let mut ring_reg: *mut quirc_region = 0 as *mut quirc_region;
    let mut ratio: i32 = 0;
    if ring_left < 0i32 || ring_right < 0i32 || stone < 0i32 {
        return;
    } else if ring_left != ring_right {
        return;
    } else if ring_left == stone {
        return;
    } else {
        stone_reg = &mut (*q).regions[stone as usize] as *mut quirc_region;
        ring_reg = &mut (*q).regions[ring_left as usize] as *mut quirc_region;
        /* Already detected */
        if (*stone_reg).capstone >= 0i32 || (*ring_reg).capstone >= 0i32 {
            return;
        } else {
            /* Ratio should ideally be 37.5 */
            ratio = (*stone_reg).count * 100i32 / (*ring_reg).count;
            if ratio < 10i32 || ratio > 70i32 {
                return;
            } else {
                record_capstone(q, ring_left, stone);
                return;
            }
        }
    };
}
unsafe extern "C" fn record_capstone(
    mut q: *mut quirc,
    mut ring: i32,
    mut stone: i32,
) -> () {
    let mut stone_reg: *mut quirc_region = &mut (*q).regions[stone as usize] as *mut quirc_region;
    let mut ring_reg: *mut quirc_region = &mut (*q).regions[ring as usize] as *mut quirc_region;
    let mut capstone: *mut quirc_capstone = 0 as *mut quirc_capstone;
    let mut cs_index: i32 = 0;
    if (*q).num_capstones >= 32i32 {
        return;
    } else {
        cs_index = (*q).num_capstones;
        let fresh4 = (*q).num_capstones;
        (*q).num_capstones = (*q).num_capstones + 1;
        capstone = &mut (*q).capstones[fresh4 as usize] as *mut quirc_capstone;
        memset(
            capstone as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<quirc_capstone>() as u64,
        );
        (*capstone).qr_grid = -1i32;
        (*capstone).ring = ring;
        (*capstone).stone = stone;
        (*stone_reg).capstone = cs_index;
        (*ring_reg).capstone = cs_index;
        /* Find the corners of the ring */
        find_region_corners(
            q,
            ring,
            &mut (*stone_reg).seed,
            (*capstone).corners.as_mut_ptr(),
        );
        /* Set up the perspective transform and find the center */
        perspective_setup(
            (*capstone).c.as_mut_ptr(),
            (*capstone).corners.as_mut_ptr(),
            7.0f64,
            7.0f64,
        );
        perspective_map(
            (*capstone).c.as_mut_ptr(),
            3.5f64,
            3.5f64,
            &mut (*capstone).center,
        );
        return;
    };
}
unsafe extern "C" fn find_region_corners(
    mut q: *mut quirc,
    mut rcode: i32,
    mut ref_0: *const quirc_point,
    mut corners: *mut quirc_point,
) -> () {
    let mut region: *mut quirc_region = &mut (*q).regions[rcode as usize] as *mut quirc_region;
    let mut psd: polygon_score_data = polygon_score_data {
        ref_0: quirc_point { x: 0, y: 0 },
        scores: [0; 4],
        corners: 0 as *mut quirc_point,
    };
    let mut i: i32 = 0;
    memset(
        &mut psd as *mut polygon_score_data as *mut libc::c_void,
        0i32,
        ::std::mem::size_of::<polygon_score_data>() as u64,
    );
    psd.corners = corners;
    memcpy(
        &mut psd.ref_0 as *mut quirc_point as *mut libc::c_void,
        ref_0 as *const libc::c_void,
        ::std::mem::size_of::<quirc_point>() as u64,
    );
    psd.scores[0usize] = -1i32;
    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        rcode,
        1i32,
        Some(find_one_corner),
        &mut psd as *mut polygon_score_data as *mut libc::c_void,
        0i32,
    );
    psd.ref_0.x = (*psd.corners.offset(0isize)).x - psd.ref_0.x;
    psd.ref_0.y = (*psd.corners.offset(0isize)).y - psd.ref_0.y;
    i = 0i32;
    while i < 4i32 {
        memcpy(
            &mut *psd.corners.offset(i as isize) as *mut quirc_point as *mut libc::c_void,
            &mut (*region).seed as *mut quirc_point as *const libc::c_void,
            ::std::mem::size_of::<quirc_point>() as u64,
        );
        i += 1
    }
    i = (*region).seed.x * psd.ref_0.x + (*region).seed.y * psd.ref_0.y;
    psd.scores[0usize] = i;
    psd.scores[2usize] = -i;
    i = (*region).seed.x * -psd.ref_0.y + (*region).seed.y * psd.ref_0.x;
    psd.scores[1usize] = i;
    psd.scores[3usize] = -i;
    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        1i32,
        rcode,
        Some(find_other_corners),
        &mut psd as *mut polygon_score_data as *mut libc::c_void,
        0i32,
    );
}
unsafe extern "C" fn find_other_corners(
    mut user_data: *mut libc::c_void,
    mut y: i32,
    mut left: i32,
    mut right: i32,
) -> () {
    let mut psd: *mut polygon_score_data = user_data as *mut polygon_score_data;
    let mut xs: [i32; 2] = [left, right];
    let mut i: i32 = 0;
    i = 0i32;
    while i < 2i32 {
        let mut up: i32 = xs[i as usize] * (*psd).ref_0.x + y * (*psd).ref_0.y;
        let mut right_0: i32 = xs[i as usize] * -(*psd).ref_0.y + y * (*psd).ref_0.x;
        let mut scores: [i32; 4] = [up, right_0, -up, -right_0];
        let mut j: i32 = 0;
        j = 0i32;
        while j < 4i32 {
            if scores[j as usize] > (*psd).scores[j as usize] {
                (*psd).scores[j as usize] = scores[j as usize];
                (*(*psd).corners.offset(j as isize)).x = xs[i as usize];
                (*(*psd).corners.offset(j as isize)).y = y
            }
            j += 1
        }
        i += 1
    }
}
unsafe extern "C" fn find_one_corner(
    mut user_data: *mut libc::c_void,
    mut y: i32,
    mut left: i32,
    mut right: i32,
) -> () {
    let mut psd: *mut polygon_score_data = user_data as *mut polygon_score_data;
    let mut xs: [i32; 2] = [left, right];
    let mut dy: i32 = y - (*psd).ref_0.y;
    let mut i: i32 = 0;
    i = 0i32;
    while i < 2i32 {
        let mut dx: i32 = xs[i as usize] - (*psd).ref_0.x;
        let mut d: i32 = dx * dx + dy * dy;
        if d > (*psd).scores[0usize] {
            (*psd).scores[0usize] = d;
            (*(*psd).corners.offset(0isize)).x = xs[i as usize];
            (*(*psd).corners.offset(0isize)).y = y
        }
        i += 1
    }
}
/* ***********************************************************************
 * Adaptive thresholding
 */
unsafe extern "C" fn threshold(mut q: *mut quirc) -> () {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut avg_w: i32 = 0i32;
    let mut avg_u: i32 = 0i32;
    let mut threshold_s: i32 = (*q).w / 8i32;
    let mut row: *mut quirc_pixel_t = (*q).pixels;
    /*
     * Ensure a sane, non-zero value for threshold_s.
     *
     * threshold_s can be zero if the image width is small. We need to avoid
     * SIGFPE as it will be used as divisor.
     */
    if threshold_s < 1i32 {
        threshold_s = 1i32
    }
    y = 0i32;
    while y < (*q).h {
        memset(
            (*q).row_average as *mut libc::c_void,
            0i32,
            ((*q).w as u64)
                .wrapping_mul(::std::mem::size_of::<i32>() as u64),
        );
        x = 0i32;
        while x < (*q).w {
            let mut w: i32 = 0;
            let mut u: i32 = 0;
            if 0 != y & 1i32 {
                w = x;
                u = (*q).w - 1i32 - x
            } else {
                w = (*q).w - 1i32 - x;
                u = x
            }
            avg_w =
                avg_w * (threshold_s - 1i32) / threshold_s + *row.offset(w as isize) as i32;
            avg_u =
                avg_u * (threshold_s - 1i32) / threshold_s + *row.offset(u as isize) as i32;
            *(*q).row_average.offset(w as isize) += avg_w;
            *(*q).row_average.offset(u as isize) += avg_u;
            x += 1
        }
        x = 0i32;
        while x < (*q).w {
            if (*row.offset(x as isize) as i32)
                < *(*q).row_average.offset(x as isize) * (100i32 - 5i32) / (200i32 * threshold_s)
            {
                *row.offset(x as isize) = 1i32 as quirc_pixel_t
            } else {
                *row.offset(x as isize) = 0i32 as quirc_pixel_t
            }
            x += 1
        }
        row = row.offset((*q).w as isize);
        y += 1
    }
}
unsafe extern "C" fn pixels_setup(mut q: *mut quirc) -> () {
    if ::std::mem::size_of::<u8>() as u64
        == ::std::mem::size_of::<quirc_pixel_t>() as u64
    {
        (*q).pixels = (*q).image
    } else {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        y = 0i32;
        while y < (*q).h {
            x = 0i32;
            while x < (*q).w {
                *(*q).pixels.offset((y * (*q).w + x) as isize) =
                    *(*q).image.offset((y * (*q).w + x) as isize);
                x += 1
            }
            y += 1
        }
    };
}
/* Extract the QR-code specified by the given index. */
#[no_mangle]
pub unsafe extern "C" fn quirc_extract(
    mut q: *const quirc,
    mut index: i32,
    mut code: *mut quirc_code,
) -> () {
    let mut qr: *const quirc_grid = &(*q).grids[index as usize] as *const quirc_grid;
    let mut y: i32 = 0;
    let mut i: i32 = 0i32;
    if index < 0i32 || index > (*q).num_grids {
        return;
    } else {
        memset(
            code as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<quirc_code>() as u64,
        );
        perspective_map(
            (*qr).c.as_ptr(),
            0.0f64,
            0.0f64,
            &mut (*code).corners[0usize],
        );
        perspective_map(
            (*qr).c.as_ptr(),
            (*qr).grid_size as f64,
            0.0f64,
            &mut (*code).corners[1usize],
        );
        perspective_map(
            (*qr).c.as_ptr(),
            (*qr).grid_size as f64,
            (*qr).grid_size as f64,
            &mut (*code).corners[2usize],
        );
        perspective_map(
            (*qr).c.as_ptr(),
            0.0f64,
            (*qr).grid_size as f64,
            &mut (*code).corners[3usize],
        );
        (*code).size = (*qr).grid_size;
        y = 0i32;
        while y < (*qr).grid_size {
            let mut x: i32 = 0;
            x = 0i32;
            while x < (*qr).grid_size {
                if read_cell(q, index, x, y) > 0i32 {
                    (*code).cell_bitmap[(i >> 3i32) as usize] =
                        ((*code).cell_bitmap[(i >> 3i32) as usize] as i32
                            | 1i32 << (i & 7i32)) as u8
                }
                i += 1;
                x += 1
            }
            y += 1
        }
        return;
    };
}
/* Read a cell from a grid using the currently set perspective
 * transform. Returns +/- 1 for black/white, 0 for cells which are
 * out of image bounds.
 */
unsafe extern "C" fn read_cell(
    mut q: *const quirc,
    mut index: i32,
    mut x: i32,
    mut y: i32,
) -> i32 {
    let mut qr: *const quirc_grid = &(*q).grids[index as usize] as *const quirc_grid;
    let mut p: quirc_point = quirc_point { x: 0, y: 0 };
    perspective_map(
        (*qr).c.as_ptr(),
        x as f64 + 0.5f64,
        y as f64 + 0.5f64,
        &mut p,
    );
    if p.y < 0i32 || p.y >= (*q).h || p.x < 0i32 || p.x >= (*q).w {
        return 0i32;
    } else {
        return if 0 != *(*q).pixels.offset((p.y * (*q).w + p.x) as isize) as i32 {
            1i32
        } else {
            -1i32
        };
    };
}
