use criterion;
use criterion::{criterion_group, criterion_main};
use image;

use rqrr;

fn bench_prepare(c: &mut criterion::Criterion) {
    let img = include_bytes!("../tests/data/full/gogh_small.jpg");
    let img = image::load_from_memory(img).unwrap().to_luma();

    c.bench_function("find_caps art", move |b| {
        b.iter_batched(|| img.clone(), |img| {
            rqrr::PreparedImage::prepare(img)
        }, criterion::BatchSize::LargeInput)
    });

}

criterion_group!(benches, bench_prepare);
criterion_main!(benches);
