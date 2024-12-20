use divan::black_box;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn part1() {
    aoc18::part1(
        black_box(include_str!("../input.txt")),
        (71, 71).into(),
        1024,
    )
    .unwrap();
}

#[divan::bench]
fn part2() {
    aoc18::part2(black_box(include_str!("../input.txt"))).unwrap();
}
