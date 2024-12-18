use divan::black_box;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    aoc17::part1(black_box(include_str!("../input.txt")));
}

#[divan::bench]
fn part2() {
    aoc17::part2(black_box(include_str!("../input.txt")));
}
