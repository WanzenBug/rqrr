use criterion;
use criterion::{criterion_group, criterion_main};
use image;

use rqrr;

fn bench_find_caps(c: &mut criterion::Criterion) {
    let img = include_bytes!("../tests/data/cap/art_small.png");
    let img = image::load_from_memory(img).unwrap().to_luma();

    let img = rqrr::PreparedImage::prepare(img);

    c.bench_function("find_caps art", move |b| {
        b.iter_batched(|| img.clone(), |mut img| {
            rqrr::capstones_from_image(&mut img)
        }, criterion::BatchSize::LargeInput)
    });
}

criterion_group!(benches, bench_find_caps);
criterion_main!(benches);
