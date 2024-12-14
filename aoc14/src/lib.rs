use std::{collections::HashMap, hash::Hash};

use glam::IVec2;
use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, separated_list1},
    sequence::separated_pair,
    IResult, Parser,
};
use nom_supreme::ParserExt;

#[derive(Debug, PartialEq)]
struct Robot {
    position: IVec2,
    velocity: IVec2,
}

fn parse_vec2(s: &str) -> IResult<&str, IVec2> {
    separated_pair(complete::i32, tag(","), complete::i32)
        .map(|(x, y)| IVec2::new(x, y))
        .parse(s)
}

fn parse_robot(s: &str) -> IResult<&str, Robot> {
    separated_pair(
        parse_vec2.preceded_by(tag("p=")),
        tag(" "),
        parse_vec2.preceded_by(tag("v=")),
    )
    .map(|(position, velocity)| Robot { position, velocity })
    .parse(s)
}

fn parse_input(s: &str) -> IResult<&str, Vec<Robot>> {
    separated_list1(line_ending, parse_robot)
        .terminated(many0(line_ending))
        .parse(s)
}

#[derive(Debug)]
struct Grid {
    x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
enum Quadrant {
    NW,
    NE,
    SE,
    SW,
}

impl Grid {
    fn move_robot(&self, r: &Robot, steps: usize) -> IVec2 {
        let s = steps as i32;
        let sx = self.x as i32;
        let sy = self.y as i32;
        IVec2::new(
            r.position.x + (r.velocity.x * (s % sx)) % sx,
            r.position.y + (r.velocity.y * (s % sy)) % sy,
        )
    }

    fn get_quadrant(&self, pos: IVec2) -> Option<Quadrant> {
        let midx = (self.x / 2) as i32;
        let midy = (self.y / 2) as i32;

        if (pos.x == midx) || (pos.y == midy) {
            return None;
        }

        Some(match (pos.x < midx, pos.y < midy) {
            (true, true) => Quadrant::NW,
            (true, false) => Quadrant::SW,
            (false, true) => Quadrant::NE,
            (false, false) => Quadrant::SE,
        })
    }
}

pub fn part1(input: &str) -> usize {
    let (r, robots) = parse_input(input).expect("valid input");
    assert!(r.is_empty());

    let g = Grid { x: 11, y: 7 };

    let mut m = HashMap::new();

    for q in robots
        .iter()
        .map(|r| g.move_robot(r, 100))
        .map(|p| g.get_quadrant(p))
    {
        m.entry(q).and_modify(|v| *v += 1).or_insert(1_usize);
    }

    m.get(&Some(Quadrant::NW)).copied().unwrap_or(0)
        * m.get(&Some(Quadrant::NE)).copied().unwrap_or(0_usize)
        * m.get(&Some(Quadrant::SW)).copied().unwrap_or(0_usize)
        * m.get(&Some(Quadrant::SE)).copied().unwrap_or(0_usize)
}

pub fn part2(input: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 12);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
