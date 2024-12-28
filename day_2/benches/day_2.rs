use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_2::day_2::{input, part_1, part_2};

fn criterion_benchmark(c: &mut Criterion) {
    let input = input(include_str!("../input.txt"));
    c.bench_function("day 2 part 1", |b| {
        b.iter(|| {
            black_box(part_1(&input.iter().copied()).unwrap());
        })
    });
    c.bench_function("day 2 part 2", |b| {
        b.iter(|| {
            black_box(part_2(&input, 19690720));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
