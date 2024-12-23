use std::collections::HashMap;

use nom::{
    character::complete::{self, line_ending},
    multi::{many0, separated_list1},
    Parser as _,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum ProcessingError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

fn parse_input(s: &str) -> Result<Vec<u32>, ProcessingError> {
    let (rest, values) = separated_list1(line_ending, complete::u32)
        .terminated(many0(line_ending))
        .parse(s)?;

    if !rest.is_empty() {
        return Err(ProcessingError::UnparsedData(rest.into()));
    }

    Ok(values)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for ProcessingError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        ProcessingError::NomError(value.map_input(|i| i.into()))
    }
}

fn next(value: u32) -> u32 {
    let value = ((value << 6) ^ value) & 0xFF_FFFF;
    let value = ((value >> 5) ^ value) & 0xFF_FFFF;

    ((value << 11) ^ value) & 0xFF_FFFF
}

fn nth_next(value: u32, cnt: u32) -> u32 {
    let mut v = value;
    for _ in 0..cnt {
        v = next(v);
    }
    v
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut cnt = 0;
    for v in input {
        cnt += nth_next(v, 2000) as usize;
    }

    Ok(cnt)
}

#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Default, Copy, Clone)]
struct Seq {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
}

impl Seq {
    fn push(&mut self, val: i32) {
        self.a = self.b;
        self.b = self.c;
        self.c = self.d;
        self.d = val;
    }

    fn delta(&self, v: u32, n: u32) -> i32 {
        ((n % 10) as i32 - (v % 10) as i32) % 10
    }

    fn push_next(&mut self, v: u32, n: u32) {
        self.push(self.delta(v, n))
    }
}

pub fn part2(input: &str) -> color_eyre::Result<u32> {
    let input = parse_input(input)?;

    let mut seq_wins = HashMap::new();

    for x in input {
        // need: all sequences of 5 numbers in the first 2000 steps
        let mut first_per_seq = HashMap::new();

        let mut seq = Seq {
            ..Default::default()
        };

        let mut v = x;
        for cnt in 0..=2000 {
            let n = next(v);
            let win = n % 10;

            seq.push_next(v, n);

            if cnt >= 3 {
                // we have enough numbers to check
                first_per_seq.entry(seq).or_insert(win);
            }
            v = n;
        }

        for (k, v) in first_per_seq.iter() {
            seq_wins.entry(*k).and_modify(|x| *x += *v).or_insert(*v);
        }
    }
    let m = *seq_wins.values().max().expect("some value");

    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();
    pub fn init_tests() {
        INIT.call_once(|| {
            color_eyre::install().unwrap_or(());
        });
    }

    #[test]
    fn test_next() {
        init_tests();
        assert_eq!(next(123), 15887950);
        assert_eq!(next(15887950), 16495136);
        assert_eq!(next(16495136), 527345);
    }

    #[test]
    fn test_part1() {
        init_tests();
        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            37327623
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example2.txt")).expect("success"), 23);
    }
}
