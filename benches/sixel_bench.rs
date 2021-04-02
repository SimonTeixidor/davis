use criterion::{criterion_group, criterion_main, Criterion};

fn bench_sixel(b: &mut Criterion) {
    let img = image::io::Reader::open("benches/lena.png")
        .unwrap()
        .decode()
        .unwrap();

    b.bench_function("to_sixel_writer", |b| {
        b.iter(|| sixel::to_sixel_writer(500, &img, 1024, std::io::sink()))
    });
}

criterion_group!(benches, bench_sixel);
criterion_main!(benches);
