use glam::IVec2;
use map_parse::Parseable;
use nom::{branch::alt, bytes::complete::tag, Parser as _};
use nom_supreme::ParserExt as _;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),

    #[error("Missing start position")]
    MissingStart,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum Cell {
    Wall,
    Start,
    End,
    Empty,
}

impl Parseable for Cell {
    type Item = Cell;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        alt((
            tag("#").value(Cell::Wall),
            tag("S").value(Cell::Start),
            tag("E").value(Cell::End),
            tag(".").value(Cell::Empty),
        ))
        .parse(s)
    }
}

struct Input {
    maze: map_parse::Map<Cell>,
    start: IVec2,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, maze) = map_parse::Map::parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    let start = maze.values_iter().find(|(_, v)| **v == Cell::Start);
    let start = match start {
        None => return Err(InputParseError::MissingStart),
        Some(value) => *value.0,
    };

    Ok(Input { maze, start })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

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
        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            11048
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
