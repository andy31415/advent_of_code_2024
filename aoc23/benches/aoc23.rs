use divan::black_box;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    aoc23::part1(black_box(include_str!("../input.txt"))).unwrap();
}

#[divan::bench]
fn part2() {
    aoc23::part2(black_box(include_str!("../input.txt"))).unwrap();
}
