use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_3::day_3::{input, part_1, part_2};

fn criterion_benchmark(c: &mut Criterion) {
    let (wire1, wire2) = input(include_str!("../input.txt"));
    c.bench_function("day 3 part 1", |b| {
        b.iter(|| {
            black_box(part_1(&wire1, &wire2));
        })
    });
    c.bench_function("day 3 part 2", |b| {
        b.iter(|| {
            black_box(part_2(&wire1, &wire2));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
