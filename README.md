## Welcome

This is my fun project for going through
[Advent of Code 2024](https://adventofcode.com/2024)

I will be using rust and will try to make reasonable code (i.e
unit testing and will have cargo and clippy passing)

### Setting up

I created a `template` subfolder to get started on a new day.

Run `cargo generate --path template` to generate a new day.
Use the project name like `aoc123` for each new day

### Running things

Just `cargo run -p aoc1` and `cargo test` is what I use the most.

- Heap profiling: `cargo run --profile dhat --features dhat-heap -p aoc2`
- Benchmarking `cargo bench`
- Flamegraph: `cargo flamegraph --profile flamegraph -p aoc2`

### Learning bits

I started this because YouTube recommended <https://youtu.be/JOgQMjpGum0?si=c4PypYS--VR5UjEU>

Implementations are vastly different, however looking at what Chris is doing
I figured I would include some things. So far what I see:

- <https://docs.rs/dhat/latest/dhat/> for heap profiling
- <https://docs.rs/divan/latest/divan/> for benchmarks

And I have not yet used but may:

- <https://docs.rs/nom/latest/nom/> - nice parsing, but for some simple strings it seems maybe
  overkill (my programs were much smaller/faster/less heap without this)

- <https://docs.rs/miette/latest/miette/> looks like a nice error reporting, never
  used (I generally go for anyhow)

I only started dhat and divan in aoc2.
