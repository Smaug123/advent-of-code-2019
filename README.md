# advent-of-code-2019
Solutions to the 2019 Advent of Code (https://adventofcode.com/2019/), in Rust.
Trying to strike a balance between performance and ease-of-writing, erring on the side of performance where necessary.

Simple to use: just `cargo test --release` or `cargo test -p day_1`, for example.
Some crates have Criterion benchmarks: `cargo bench`, or `cargo bench -p day_1`, for example.

I'm certainly no expert in Rust; don't assume I've done anything in a sane way.

## How to use

To run tests on real inputs, create `inputs/day_1.txt` (for example) at the top level.
To run tests without real inputs available, build with the feature `no_real_inputs` enabled.
