use std::collections::{HashSet, VecDeque};

use glam::IVec2;
use map_parse::Parseable;
use nom::{character::complete::satisfy, Parser};

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Eq, Ord)]
enum Plant {
    Value(char),
}

impl Parseable for Plant {
    type Item = Plant;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        satisfy(|c| c != '\r' && c != '\n')
            .map(Plant::Value)
            .parse(s)
    }
}

fn p1_cost(
    start_position: &IVec2,
    garden: &map_parse::Map<Plant>,
    seen: &mut HashSet<IVec2>,
) -> usize {
    if seen.contains(start_position) {
        return 0;
    }

    let plant = garden
        .get(start_position)
        .expect("Starting position is valid");

    let mut to_check = VecDeque::new();

    seen.insert(*start_position);
    to_check.push_back(*start_position);

    let mut walls = 0;
    let mut cells = 0;

    while let Some(value) = to_check.pop_front() {
        cells += 1;

        for dir in [
            IVec2::new(0, 1),
            IVec2::new(0, -1),
            IVec2::new(1, 0),
            IVec2::new(-1, 0),
        ] {
            let new_pos = value + dir;

            if !garden.is_inside(new_pos) {
                walls += 1;
                continue;
            }

            if garden.get(&new_pos).expect("valid point") == plant {
                if !seen.contains(&new_pos) {
                    seen.insert(new_pos);
                    to_check.push_back(new_pos);
                }
            } else {
                walls += 1;
            }
        }
    }
    tracing::info!(
        "Starting at {:?} cell {:?} has {} walls and {} cells",
        start_position,
        plant,
        walls,
        cells
    );
    walls * cells
}

pub fn part1(input: &str) -> usize {
    let (r, m) = map_parse::Map::<Plant>::parse(input).expect("Valid input");
    assert!(r.is_empty());

    let mut seen = HashSet::new();

    m.values_iter()
        .map(|(pos, _)| p1_cost(pos, &m, &mut seen))
        .sum()
}

pub fn part2(input: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 1930);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
