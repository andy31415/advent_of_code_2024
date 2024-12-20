use std::{collections::HashSet, hash::Hash};

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
use pathfinding::prelude::{dijkstra, dijkstra_all};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct RacePosition {
    pos: IVec2,
    cheat: Option<(IVec2, IVec2)>,
}

impl RacePosition {
    fn without_cheats(pos: IVec2) -> Self {
        Self { pos, cheat: None }
    }

    fn plain_successors(
        &self,
        walls: &HashSet<IVec2>,
        rows: usize,
        cols: usize,
    ) -> Vec<RacePosition> {
        [(0, 1), (0, -1), (1, 0), (-1, 0)]
            .into_iter()
            .map(|(x, y)| self.pos + IVec2::new(x, y))
            .filter(|p| {
                p.x >= 0
                    && (p.x as usize) < cols
                    && p.y >= 0
                    && (p.y as usize) < rows
                    && !walls.contains(p)
            })
            .map(|pos| RacePosition {
                pos,
                cheat: self.cheat,
            })
            .collect::<Vec<_>>()
    }

    fn cheat_successors(
        &self,
        walls: &HashSet<IVec2>,
        rows: usize,
        cols: usize,
        banned_cheats: &HashSet<(IVec2, IVec2)>,
    ) -> Vec<RacePosition> {
        let mut result = Vec::new();
        for pos in [(0, 1), (0, -1), (1, 0), (-1, 0)]
            .into_iter()
            .map(|(x, y)| self.pos + IVec2::new(x, y))
            .filter(|p| p.x >= 0 && (p.x as usize) < cols && p.y >= 0 && (p.y as usize) < rows)
        {
            if !walls.contains(&pos) {
                result.push(RacePosition {
                    pos,
                    cheat: self.cheat,
                });
            } else {
                // this is inside a wall ... can we cheat?
                // attempt to move double and see what happens
                if self.cheat.is_some() {
                    continue;
                }

                // teleport-pos
                let end_pos = self.pos + (self.pos - pos);
                if walls.contains(&end_pos) {
                    continue;
                }

                if banned_cheats.contains(&(self.pos, end_pos)) {
                    continue; // cheat already considered
                }
                println!("CAN CHEAT {} TO {}", pos, end_pos);

                result.push(RacePosition {
                    pos: end_pos,
                    cheat: Some((self.pos, end_pos)),
                });
            }
        }
        result
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let start_cost = dijkstra(
        &RacePosition::without_cheats(input.start),
        |start| {
            start
                .plain_successors(&input.walls, input.rows, input.cols)
                .iter()
                .map(|p| (*p, p.cheat.map(|c| 2).unwrap_or(1)))
                .collect::<Vec<_>>()
        },
        |x| x.pos == input.end,
    )
    .expect("Has path")
    .1;

    tracing::info!("START COST: {}", start_cost);

    let mut banned_cheats = HashSet::new();
    let mut saves = 0;

    loop {
        println!("SEARCHING...");
        // see what happens if we cheat
        let path = dijkstra(
            &RacePosition::without_cheats(input.start),
            |start| {
                start
                    .cheat_successors(&input.walls, input.rows, input.cols, &banned_cheats)
                    .iter()
                    .map(|p| (*p, 1))
                    .collect::<Vec<_>>()
            },
            |x| x.pos == input.end,
        )
        .expect("Has path");

        println!(
            "FOUND CHEAT PATH length : {:#?} (saving {})",
            path.1,
            start_cost - path.1
        );

        // if start_cost - path.1 <= 100 {
        //     break;
        // }
        if start_cost - path.1 == 0 {
            break;
        }

        println!(
            "FOUND CHEAT PATH length : {:#?} (saving {})",
            path.1,
            start_cost - path.1
        );
        saves += 1;

        // ban the cheat used
        for p in path.0.iter().filter_map(|p| p.cheat) {
            banned_cheats.insert(p);
        }
    }

    Ok(saves)
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

    Ok(0)
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
