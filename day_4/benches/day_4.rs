use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_4::day_4::{input, part_1, part_2};

fn criterion_benchmark(c: &mut Criterion) {
    let (low, high) = input(include_str!("../input.txt"));
    c.bench_function("day 4 part 1", |b| {
        b.iter(|| {
            black_box(part_1(low, high));
        })
    });
    c.bench_function("day 4 part 2", |b| {
        b.iter(|| {
            black_box(part_2(low, high));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
