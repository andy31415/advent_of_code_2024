use glam::IVec2;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
struct Map {
    rows: i32,
    cols: i32,
    antennas: HashMap<char, HashSet<IVec2>>, // col, row (aka (x, y))
}

impl Map {
    fn contains(&self, location: IVec2) -> bool {
        (location.x >= 0)
            && (location.x < self.cols)
            && (location.y >= 0)
            && (location.y < self.rows)
    }
}

pub mod parsing {
    use std::collections::HashSet;

    use crate::Map;
    use glam::IVec2;
    use nom::{
        character::complete::{line_ending, none_of},
        multi::many1,
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    /// find the antenna: gives a character id and the X position
    fn maybe_antenna(input: &str) -> IResult<&str, Option<char>> {
        none_of("\r\n")
            .map(|c| match c {
                '.' => None,
                a => Some(a),
            })
            .parse(input)
    }

    #[tracing::instrument]
    pub fn map(input: &str) -> IResult<&str, Map> {
        many1(many1(maybe_antenna).terminated(line_ending))
            .map(|antennas| {
                let mut map = Map {
                    rows: antennas.len() as i32,
                    cols: antennas.first().expect("non-empty map").len() as i32,
                    ..Default::default()
                };

                antennas.iter().enumerate().for_each(|(row, row_vec)| {
                    assert_eq!(row_vec.len(), map.cols as usize);
                    row_vec
                        .iter()
                        .enumerate()
                        .filter_map(|(col, maybe_antenna)| maybe_antenna.map(|c| (c, col)))
                        .for_each(|(id, col)| {
                            let antenna_pos = IVec2::new(col as i32, row as i32);
                            map.antennas
                                .entry(id)
                                .and_modify(|h| {
                                    h.insert(antenna_pos);
                                })
                                .or_insert_with(|| {
                                    let mut h = HashSet::new();
                                    h.insert(antenna_pos);
                                    h
                                });
                        })
                });
                map
            })
            .parse(input)
    }
}

pub fn part1(input: &str) -> usize {
    let (r, map) = parsing::map(input).expect("valid input");
    assert!(r.is_empty());

    let mut antinodes = HashSet::new();

    map.antennas.iter().for_each(|(_, positions)| {
        // have to combine every position with every other position.
        positions.iter().tuple_combinations().for_each(|(p1, p2)| {
            if p1 == p2 {
                return;
            }
            let p = *p1 + *p1 - *p2;
            if map.contains(p) {
                antinodes.insert(p);
            }

            let p = *p2 + *p2 - *p1;
            if map.contains(p) {
                antinodes.insert(p);
            }
        });
    });

    antinodes.len()
}

pub fn part2(input: &str) -> usize {
    let (r, map) = parsing::map(input).expect("valid input");
    assert!(r.is_empty());

    let mut antinodes = HashSet::new();

    map.antennas.iter().for_each(|(_, positions)| {
        // have to combine every position with every other position.
        positions.iter().tuple_combinations().for_each(|(p1, p2)| {
            if p1 == p2 {
                return;
            }

            let d = p1 - p2;
            let mut p = p1 + d;
            while map.contains(p) {
                antinodes.insert(p);
                p += d;
            }

            let d = p2 - p1;
            let mut p = p2 + d;
            while map.contains(p) {
                antinodes.insert(p);
                p += d;
            }

            // also add one on the antenna
            antinodes.insert(*p1);
            antinodes.insert(*p2);
        })
    });

    antinodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 14);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 34);
    }
}
