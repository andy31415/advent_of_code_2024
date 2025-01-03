use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use glam::IVec2;
use nom::{branch::alt, bytes::complete::tag, Parser};
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
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let start_cost = dijkstra(
        &RacePosition::without_cheats(input.start),
        |start| {
            start
                .plain_successors(&input.walls, input.rows, input.cols)
                .iter()
                .map(|p| (*p, 1))
                .collect::<Vec<_>>()
        },
        |x| x.pos == input.end,
    )
    .expect("Has path")
    .1;

    tracing::info!("START COST: {}", start_cost);

    // logic: find distance to start for ALL walls
    let mut distance_from_start = HashMap::new();
    let paths = dijkstra_all(&RacePosition::without_cheats(input.start), |start| {
        start
            .plain_successors(&input.walls, input.rows, input.cols)
            .iter()
            .map(|p| (*p, 1))
            .collect::<Vec<_>>()
    });

    for (pos, (_, len)) in paths.iter() {
        distance_from_start.insert(pos.pos, *len);
    }
    distance_from_start.insert(input.start, 0);

    tracing::info!("Costs calculated calculated!");

    let mut cnt = 0;

    // For every empty space that is near a wall, figure out what to do
    for x in 0..input.cols as i32 {
        for y in 0..input.rows as i32 {
            let start = IVec2::new(x, y);
            if input.walls.contains(&start) {
                continue;
            }

            // at this point, we are allowed to cheat in mahattan distance of up to 20
            // so this is quite rough...
            for dx in -2..=2 {
                for dy in -2..=2 {
                    let d = IVec2::new(dx, dy);

                    if dx.unsigned_abs() + dy.unsigned_abs() > 2 {
                        // is this a valid cheat ?
                        continue;
                    }
                    // go to the end, can do walls
                    let end = start + d;
                    if input.walls.contains(&end) {
                        continue;
                    }
                    if end.x < 0
                        || end.x >= input.cols as i32
                        || end.y < 0
                        || end.y >= input.rows as i32
                    {
                        continue;
                    }

                    let d_start = match distance_from_start.get(&start) {
                        Some(value) => value,
                        None => {
                            tracing::error!("UNEXPECTED NO DISTANCE FOR {}", start);
                            continue;
                        }
                    };
                    let d_end = match distance_from_start.get(&end) {
                        Some(value) => value,
                        None => {
                            tracing::error!("UNEXPECTED NO DISTANCE FOR {}", end);
                            continue;
                        }
                    };

                    let saving = (d_end - d_start) - dx.abs() - dy.abs();

                    if saving >= 100 {
                        tracing::info!("CHECK CHEAT {} -> {}: {}", start, end, saving);
                        cnt += 1;
                    }
                }
            }
        }
    }

    Ok(cnt)
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let start_cost = dijkstra(
        &RacePosition::without_cheats(input.start),
        |start| {
            start
                .plain_successors(&input.walls, input.rows, input.cols)
                .iter()
                .map(|p| (*p, 1))
                .collect::<Vec<_>>()
        },
        |x| x.pos == input.end,
    )
    .expect("Has path")
    .1;

    tracing::info!("START COST: {}", start_cost);

    // logic: find distance to start for ALL walls
    let mut distance_from_start = HashMap::new();
    let paths = dijkstra_all(&RacePosition::without_cheats(input.start), |start| {
        start
            .plain_successors(&input.walls, input.rows, input.cols)
            .iter()
            .map(|p| (*p, 1))
            .collect::<Vec<_>>()
    });

    for (pos, (_, len)) in paths.iter() {
        distance_from_start.insert(pos.pos, *len);
    }
    distance_from_start.insert(input.start, 0);

    tracing::info!("Costs calculated calculated!");

    let mut cnt = 0;

    // For every empty space that is near a wall, figure out what to do
    for x in 0..input.cols as i32 {
        for y in 0..input.rows as i32 {
            let start = IVec2::new(x, y);
            if input.walls.contains(&start) {
                continue;
            }

            // at this point, we are allowed to cheat in mahattan distance of up to 20
            // so this is quite rough...
            for dx in -20..=20 {
                for dy in -20..=20 {
                    let d = IVec2::new(dx, dy);

                    if dx.abs() + dy.abs() > 20 {
                        // is this a valid cheat ?
                        continue;
                    }
                    // go to the end, can do walls
                    let end = start + d;
                    if input.walls.contains(&end) {
                        continue;
                    }
                    if end.x < 0
                        || end.x >= input.cols as i32
                        || end.y < 0
                        || end.y >= input.rows as i32
                    {
                        continue;
                    }

                    let d_start = match distance_from_start.get(&start) {
                        Some(value) => value,
                        None => {
                            tracing::error!("UNEXPECTED NO DISTANCE FOR {}", start);
                            continue;
                        }
                    };
                    let d_end = match distance_from_start.get(&end) {
                        Some(value) => value,
                        None => {
                            tracing::error!("UNEXPECTED NO DISTANCE FOR {}", end);
                            continue;
                        }
                    };

                    let saving = (d_end - d_start) - dx.abs() - dy.abs();

                    if saving >= 100 {
                        tracing::info!("CHECK CHEAT {} -> {}: {}", start, end, saving);
                        cnt += 1;
                    }
                }
            }
        }
    }

    Ok(cnt)
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

    //#[test_log::test]
    #[test]
    fn test_part1() {
        init_tests();
        assert_eq!(part1(include_str!("../example.txt")).expect("success"), 0);
    }

    #[test_log::test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
