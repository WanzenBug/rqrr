use crate::CapStone;

#[derive(Debug, Clone)]
pub struct CapStoneGroup(pub CapStone, pub CapStone, pub CapStone);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Neighbor {
    index: usize,
    distance: f64,
}

/// Return each pair Capstone indexes that are likely to be from a QR code
/// Ordered from most symmetric to least symmetric
pub fn find_and_rank_possible_neighbors(capstones: &[CapStone], idx: usize) -> Vec<(usize, usize)> {
    const VIABILITY_THRESHOLD: f64 = 0.25;

    let (hlist, vlist) = find_possible_neighbors(capstones, idx);
    let mut res = Vec::new();
    struct NeighborSet {
        score: f64,
        h_index: usize,
        v_index: usize,
    }
    /* Test each possible grouping */
    for hn in hlist {
        for vn in vlist.iter() {
            let score = {
                if hn.distance < vn.distance {
                    (1.0f64 - hn.distance / vn.distance).abs()
                } else {
                    (1.0f64 - vn.distance / hn.distance).abs()
                }
            };
            if score < VIABILITY_THRESHOLD {
                res.push(NeighborSet {
                    score,
                    h_index: hn.index,
                    v_index: vn.index,
                });
            }
        }
    }

    res.sort_unstable_by(|a, b| {
        (a.score)
            .partial_cmp(&(b.score))
            .expect("Neighbor distance should never cause a div by 0")
    });
    res.iter().map(|n| (n.h_index, n.v_index)).collect()
}

fn find_possible_neighbors(capstones: &[CapStone], idx: usize) -> (Vec<Neighbor>, Vec<Neighbor>) {
    let cap = &capstones[idx];
    let mut hlist = Vec::new();
    let mut vlist = Vec::new();

    /* Look for potential neighbours by examining the relative gradients
     * from this capstone to others.
     */
    #[allow(clippy::needless_range_loop)]
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
