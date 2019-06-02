use crate::CapStone;

#[derive(Debug, Clone)]
pub struct CapStoneGroup(
    pub CapStone,
    pub CapStone,
    pub CapStone
);


#[derive(Clone, Copy, Debug, PartialEq)]
struct Neighbor {
    index: usize,
    distance: f64,
}

/// Find CapStones that form a grid
///
/// By trying to match up the relative perspective of 3 [CapStones](struct.CapStone.html) we can
/// find those that corner the same QR code.
pub fn find_groupings(mut capstones: Vec<CapStone>) -> Vec<CapStoneGroup> {
    let mut idx = 0;
    let mut groups = Vec::new();
    while idx < capstones.len() {
        let (hlist, vlist) = find_possible_neighbors(&capstones, idx);
        match test_neighbours(&hlist, &vlist) {
            None => {
                idx += 1
            }
            Some((h_idx, v_idx)) => {
                let group = remove_capstones_in_order(&mut capstones,
                                                      h_idx,
                                                      idx,
                                                      v_idx);
                groups.push(group);

                // Update index for items removed
                let sub = [h_idx, v_idx].iter()
                    .filter(|&&i| i < idx)
                    .count();
                idx -= sub;
            }
        }
    }

    groups
}

fn remove_capstones_in_order(caps: &mut Vec<CapStone>,
                             first: usize,
                             second: usize,
                             third: usize) -> CapStoneGroup {
    assert_ne!(first, second);
    assert_ne!(first, third);
    assert_ne!(second, third);

    let idx0 = first;
    let mut idx1 = second;
    let mut idx2 = third;

    if second > first {
        idx1 -= 1;
    }

    if third > first {
        idx2 -= 1;
    }

    if third > second {
        idx2 -= 1;
    }

    let first_cap = caps.remove(idx0);
    let second_cap = caps.remove(idx1);
    let third_cap = caps.remove(idx2);

    CapStoneGroup(first_cap, second_cap, third_cap)
}

fn find_possible_neighbors(capstones: &[CapStone], idx: usize) -> (Vec<Neighbor>, Vec<Neighbor>) {
    let cap = &capstones[idx];
    let mut hlist = Vec::new();
    let mut vlist = Vec::new();

    /* Look for potential neighbours by examining the relative gradients
     * from this capstone to others.
     */
    for others_idx in 0..capstones.len() {
        if others_idx == idx {
            continue;
        }

        let cmp_cap = &capstones[others_idx];

        let (mut u, mut v) = cap.c.unmap(&cmp_cap.center);
        u = (u - 3.5f64).abs();
        v = (v - 3.5f64).abs();

        if u < 0.2f64 * v {
            hlist.push(Neighbor {
                index: others_idx,
                distance: v,
            });
        }

        if v < 0.2f64 * u {
            vlist.push(Neighbor {
                index: others_idx,
                distance: u,
            });
        }
    }

    (hlist, vlist)
}

fn test_neighbours(
    hlist: &[Neighbor],
    vlist: &[Neighbor],
) -> Option<(usize, usize)> {
    // Worse scores will be ignored anyway
    let mut best_score = 2.5;
    let mut best_h = None;
    let mut best_v = None;
    /* Test each possible grouping */
    for hn in hlist {
        for vn in vlist {
            let score = (1.0f64 - hn.distance / vn.distance).abs();

            if score > 2.5 {
                continue;
            }

            let new = match (best_h, score) {
                (None, _) => {
                    (Some(hn.index), Some(vn.index), score)
                }
                (Some(_), b) if b < best_score => {
                    (Some(hn.index), Some(vn.index), b)
                }
                _ => (best_h, best_v, best_score)
            };

            best_h = new.0;
            best_v = new.1;
            best_score = new.2;
        }
    }

    match (best_h, best_v) {
        (None, _)
        | (_, None) => None,
        (Some(h), Some(v)) => Some((h, v))
    }
}
