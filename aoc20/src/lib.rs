use std::collections::HashSet;

use glam::IVec2;
use nom::{branch::alt, bytes::complete::tag, character::complete::satisfy, Parser};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}
use pathfinding::prelude::dijkstra;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug)]
enum Cell {
    Wall,
    Empty,
    Start,
    End,
}

impl map_parse::Parseable for Cell {
    type Item = Cell;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        alt((
            tag("#").value(Cell::Wall),
            tag(".").value(Cell::Empty),
            tag("S").value(Cell::Start),
            tag("E").value(Cell::End),
        ))
        .parse(s)
    }
}

struct Input {
    start: IVec2,
    end: IVec2,
    walls: HashSet<IVec2>,
    rows: usize,
    cols: usize,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, map) = map_parse::Map::<Cell>::parse(s)?;
    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    let start = map
        .values_iter()
        .find(|(_, c)| **c == Cell::Start)
        .map(|(p, _)| *p)
        .expect("has start");
    let end = map
        .values_iter()
        .find(|(_, c)| **c == Cell::End)
        .map(|(p, _)| *p)
        .expect("has start");
    let walls = map
        .values_iter()
        .filter(|(_, c)| **c == Cell::Wall)
        .map(|(p, _)| *p)
        .collect();

    Ok(Input {
        start,
        end,
        walls,
        rows: map.row_count(),
        cols: map.col_count(),
    })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

    let start_cost = dijkstra(
        &input.start,
        |start| {
            [(0, 1), (0, -1), (1, 0), (-1, 0)]
                .into_iter()
                .map(|(x, y)| start + IVec2::new(x, y))
                .filter(|p| {
                    p.x >= 0 && (p.x as usize) < input.cols && p.y >= 0 && (p.y as usize) < input.rows && !input.walls.contains(p)
                })
                .map(|p| (p, 1))
                .collect::<Vec<_>>()
        },
        |x| x == &input.end,
    )
    .expect("Has path")
    .1;

    println!("START COST: {}", start_cost);
    // FIXME: dijskstra first, figure out cost

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
