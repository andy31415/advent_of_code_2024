use std::{collections::HashMap, hash::Hash};

use glam::IVec2;
use nom::{
    bytes::complete::is_a,
    character::complete::line_ending,
    multi::{many0, separated_list1},
    Parser,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

#[derive(Debug)]
struct Input {
    inputs: Vec<String>,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, inputs) = separated_list1(
        line_ending,
        is_a("0123456789A").map(|s: &str| s.to_string()),
    )
    .terminated(many0(line_ending))
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(Input { inputs })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

struct KeyPad {
    key_coord: HashMap<char, IVec2>,
    remote_coord: HashMap<char, IVec2>,
}

impl KeyPad {
    fn new() -> Self {
        let mut key_coord = HashMap::new();

        key_coord.insert('7', (0, 0).into());
        key_coord.insert('8', (1, 0).into());
        key_coord.insert('9', (2, 0).into());
        key_coord.insert('4', (0, 1).into());
        key_coord.insert('5', (1, 1).into());
        key_coord.insert('6', (2, 2).into());
        key_coord.insert('1', (0, 2).into());
        key_coord.insert('2', (1, 2).into());
        key_coord.insert('3', (2, 2).into());
        key_coord.insert('0', (1, 3).into());
        key_coord.insert('A', (2, 3).into());

        let mut remote_coord = HashMap::new();

        remote_coord.insert('^', (1, 0).into());
        remote_coord.insert('A', (2, 0).into());
        remote_coord.insert('<', (0, 1).into());
        remote_coord.insert('v', (1, 1).into());
        remote_coord.insert('>', (2, 1).into());

        Self {
            key_coord,
            remote_coord,
        }
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let keypad = KeyPad::new();

    println!("INPUT: {:#?}", input);

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
