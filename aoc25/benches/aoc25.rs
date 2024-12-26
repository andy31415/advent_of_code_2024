use divan::black_box;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    aoc25::part1(black_box(include_str!("../input.txt"))).unwrap();
}
