use std::{
    collections::HashSet,
    fmt::{Display, Write},
};

use glam::IVec2;
use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, separated_list1},
    sequence::separated_pair,
    Parser as _,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

struct Input {
    positions: Vec<IVec2>,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, input) = separated_list1(
        line_ending,
        separated_pair(complete::i32, tag(","), complete::i32).map(|(x, y)| IVec2::new(x, y)),
    )
    .terminated(many0(line_ending))
    .map(|positions| Input { positions })
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(input)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

struct Grid {
    rows: i32,
    cols: i32,
    blocks: HashSet<IVec2>,
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.rows {
            let mut s = String::with_capacity((self.cols + 1) as usize);
            for x in 0..self.cols {
                if self.blocks.contains(&(x, y).into()) {
                    s.push('#');
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
            f.write_str(&s)?;
        }
        Ok(())
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

    // GRID SIZE: 6x6 OR 70x70
    let g = Grid {
        rows: 7,
        cols: 7,
        blocks: input.positions.into_iter().take(12).collect(),
    };

    println!("GRID:\n{}", g);

    todo!();
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

    todo!();
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
    fn test_part1() {
        init_tests();
        assert_eq!(part1(include_str!("../example.txt")).expect("success"), 0);
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
