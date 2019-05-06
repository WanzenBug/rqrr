use rqrr;
use image;
use std::time::Instant;
use std::collections::HashSet;


#[test]
fn test_full_time() {
    let jpg = image::open("tests/data/full/gogh.jpg").unwrap();

    let now = Instant::now();
    let mut search_img = rqrr::SearchableImage::from_dynamic(&jpg);
    println!("leveling: {}ms", now.elapsed().as_millis());

    let now = Instant::now();
    let caps = rqrr::capstones_from_image(&mut search_img);

    println!("capstones: {}ms", now.elapsed().as_millis());
    assert!(9 <= caps.len());

    let now = Instant::now();
    let groups = rqrr::find_groupings(caps);
    println!("groups: {}ms", now.elapsed().as_millis());
    assert_eq!(3, groups.len());

    let mut codes = HashSet::new();
    for group in groups {
        let now = Instant::now();
        let location = rqrr::SkewedGridLocation::from_group(&mut search_img, group).unwrap();
        println!("location setup: {}ms", now.elapsed().as_millis());

        let grid = location.into_grid_image(&search_img);

        let mut buf = Vec::new();
        let now = Instant::now();
        rqrr::decode(&grid, &mut buf).unwrap();
        println!("decode: {}ms", now.elapsed().as_millis());
        codes.insert(String::from_utf8(buf).unwrap());
    }

    let mut ref_set = HashSet::new();
    ref_set.insert("https://github.com/WanzenBug/rqrr".to_string());
    ref_set.insert("rqrr".to_string());
    ref_set.insert("1234567891011121314151617181920".to_string());
    assert_eq!(ref_set, codes);
}
