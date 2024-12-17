use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

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
    end: IVec2,
}

impl Input {
    fn compute_walk_back_count(
        &self,
        position: IVec2,
        direction: Direction,
        route_costs: &HashMap<(IVec2, Direction), usize>,
        seen: &mut HashSet<IVec2>,
    ) {
        seen.insert(position);

        if position == self.start {
            return;
        }

        // figure out what the current cost is
        let current_cost = route_costs.get(&(position, direction)).expect("has cost");

        // we can "walk back" or turn. See what works
        let choices = [
            (position - direction.vec(), direction, current_cost - 1),
            (position, direction.turn_left(), current_cost - 1000),
            (position, direction.turn_right(), current_cost - 1000),
        ];

        for (p, d, c) in choices {
            if route_costs.get(&(p, d)) == Some(&c) {
                self.compute_walk_back_count(p, d, route_costs, seen);
            }
        }
    }
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

    let end = maze.values_iter().find(|(_, v)| **v == Cell::End);
    let end = match end {
        None => return Err(InputParseError::MissingStart),
        Some(value) => *value.0,
    };

    Ok(Input { maze, start, end })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum Direction {
    E,
    N,
    S,
    W,
}

impl Direction {
    fn vec(&self) -> IVec2 {
        match self {
            Direction::E => IVec2::new(1, 0),
            Direction::N => IVec2::new(0, -1),
            Direction::S => IVec2::new(0, 1),
            Direction::W => IVec2::new(-1, 0),
        }
    }

    fn turn_left(&self) -> Direction {
        match self {
            Direction::E => Direction::N,
            Direction::N => Direction::W,
            Direction::S => Direction::E,
            Direction::W => Direction::S,
        }
    }

    fn turn_right(&self) -> Direction {
        match self {
            Direction::E => Direction::S,
            Direction::N => Direction::E,
            Direction::S => Direction::W,
            Direction::W => Direction::N,
        }
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    // fill up the cost from the start
    let mut to_check = VecDeque::new();
    let mut route_costs = HashMap::new();

    route_costs.insert((input.start, Direction::E), 0);
    to_check.push_back((input.start, Direction::E, 0));

    while let Some((pos, heading, cost)) = to_check.pop_front() {
        let next_choices = [
            (pos + heading.vec(), heading, cost + 1),
            (pos, heading.turn_left(), cost + 1000),
            (pos, heading.turn_right(), cost + 1000),
        ];

        for (next_pos, next_heading, next_cost) in next_choices {
            if input.maze.get(&next_pos).unwrap_or(&Cell::Wall) != &Cell::Wall
                && match route_costs.get(&(next_pos, next_heading)) {
                    None => true,
                    Some(value) if *value > next_cost => true,
                    _ => false,
                }
            {
                to_check.push_back((next_pos, next_heading, next_cost));
                route_costs.insert((next_pos, next_heading), next_cost);
            }
        }
    }

    Ok(**[
        route_costs
            .get(&(input.end, Direction::N))
            .expect("Route exists"),
        route_costs
            .get(&(input.end, Direction::E))
            .expect("Route exists"),
        route_costs
            .get(&(input.end, Direction::S))
            .expect("Route exists"),
        route_costs
            .get(&(input.end, Direction::W))
            .expect("Route exists"),
    ]
    .iter()
    .min()
    .expect("have values"))
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    // fill up the cost from the start
    let mut to_check = VecDeque::new();
    let mut route_costs = HashMap::new();

    route_costs.insert((input.start, Direction::E), 0);
    to_check.push_back((input.start, Direction::E, 0));

    while let Some((pos, heading, cost)) = to_check.pop_front() {
        let next_choices = [
            (pos + heading.vec(), heading, cost + 1),
            (pos, heading.turn_left(), cost + 1000),
            (pos, heading.turn_right(), cost + 1000),
        ];

        for (next_pos, next_heading, next_cost) in next_choices {
            if input.maze.get(&next_pos).unwrap_or(&Cell::Wall) == &Cell::Wall {
                continue;
            }

            if match route_costs.get(&(next_pos, next_heading)) {
                None => true,
                Some(value) if *value > next_cost => true,
                _ => false,
            } {
                to_check.push_back((next_pos, next_heading, next_cost));
                route_costs.insert((next_pos, next_heading), next_cost);
            }
        }
    }

    let mut best_direction = Direction::N;
    let mut best_cost = route_costs
        .get(&(input.end, Direction::N))
        .expect("has value");

    for d in [Direction::S, Direction::E, Direction::W] {
        let c = route_costs.get(&(input.end, d)).expect("has value");
        if c < best_cost {
            best_cost = c;
            best_direction = d;
        }
    }

    let mut seen = HashSet::new();

    input.compute_walk_back_count(input.end, best_direction, &route_costs, &mut seen);

    Ok(seen.len())
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
            7036
        );

        assert_eq!(
            part1(include_str!("../example2.txt")).expect("success"),
            11048
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 45);
        assert_eq!(part2(include_str!("../example2.txt")).expect("success"), 64);
    }
}
