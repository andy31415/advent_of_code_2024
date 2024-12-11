use std::collections::HashSet;

use glam::IVec2;
use map_parse::{Map, Parseable};
use nom::{character::complete::satisfy, Parser};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Height {
    Value(u8),
}

enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn vec(self) -> IVec2 {
        match self {
            Direction::N => IVec2::new(0, -1),
            Direction::E => IVec2::new(1, 0),
            Direction::S => IVec2::new(0, 1),
            Direction::W => IVec2::new(-1, 0),
        }
    }
}

impl Parseable for Height {
    type Item = Height;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        satisfy(|c| c.is_ascii_digit())
            .map(|c| Height::Value(c.to_digit(10).expect("it is a digit") as u8))
            .parse(s)
    }
}

fn path_count_to_9(map: &Map<Height>, start: &IVec2) -> usize {
    let mut positions = HashSet::new();
    let mut seen = HashSet::new();
    let mut ends = HashSet::new();

    positions.insert(*start);
    seen.insert(*start);

    while !positions.is_empty() {
        let mut next_positions = HashSet::new();
        seen.extend(positions.iter().copied());

        for p in positions {
            let Height::Value(current_value) = map.get(&p).expect("Position is valid");
            let next_value = current_value + 1;
            for d in [Direction::N, Direction::E, Direction::S, Direction::W] {
                let next_pos = p + d.vec();

                if !map.is_inside(next_pos) {
                    continue;
                }
                if seen.contains(&next_pos) {
                    continue;
                }

                match map.get(&next_pos) {
                    Some(Height::Value(v)) if *v == next_value => {
                        if *v == 9 {
                            ends.insert(next_pos);
                        } else {
                            next_positions.insert(next_pos);
                        }
                    }
                    _ => {}
                }
            }
        }
        positions = next_positions;
    }

    ends.len()
}

fn trail_head_rating(map: &Map<Height>, start: &IVec2) -> usize {
    let Height::Value(current) = map.get(start).expect("valid position");

    if *current >= 9 {
        return 1; // found a trail end
    }
    let next_value = current + 1;

    let mut cnt = 0;

    for d in [Direction::N, Direction::E, Direction::S, Direction::W] {
        let next_pos = start + d.vec();

        if !map.is_inside(next_pos) {
            continue;
        }

        match map.get(&next_pos) {
            Some(Height::Value(v)) if *v == next_value => cnt += trail_head_rating(map, &next_pos),
            _ => {}
        }
    }

    cnt
    // recursive search for all values
}

pub fn part1(input: &str) -> usize {
    let (r, m) = Map::<Height>::parse(input).expect("valid input");
    assert!(r.is_empty());

    // Find all the 0
    m.values_iter()
        .filter(|(_, value)| **value == Height::Value(0))
        .map(|(pos, _)| path_count_to_9(&m, pos))
        .sum()
}

pub fn part2(input: &str) -> usize {
    let (r, m) = Map::<Height>::parse(input).expect("valid input");
    assert!(r.is_empty());

    // Find all the 0
    m.values_iter()
        .filter(|(_, value)| **value == Height::Value(0))
        .map(|(pos, _)| trail_head_rating(&m, pos))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 36);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 81);
    }
}
