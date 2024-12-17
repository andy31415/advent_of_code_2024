use divan::black_box;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    aoc16::part1(black_box(include_str!("../input.txt")));
}

#[divan::bench]
fn part2() {
    aoc16::part2(black_box(include_str!("../input.txt")));
}
