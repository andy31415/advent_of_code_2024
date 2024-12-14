use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use glam::IVec2;
use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, separated_list1},
    sequence::separated_pair,
    IResult, Parser,
};
use nom_supreme::ParserExt;
use rayon::prelude::*;

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
    #[tracing::instrument(ret, level = "trace")]
    fn move_robot(&self, r: &Robot, steps: usize) -> IVec2 {
        tracing::debug!("MOVING");

        let s = steps as i32;

        IVec2::new(
            (r.position.x + r.velocity.x * s).rem_euclid(self.x as i32),
            (r.position.y + r.velocity.y * s).rem_euclid(self.y as i32),
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

    fn display_robots(&self, v: &HashSet<IVec2>, steps: usize) {
        let mut s = String::with_capacity((self.x + 1) * self.y);
        for y in 0..self.y {
            for x in 0..self.x {
                if v.contains(&IVec2::new(x as i32, y as i32)) {
                    s.push('*');
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
        }
        println!("{}\nSTEPS: {}", s, steps);
    }
}

pub fn part1(input: &str) -> usize {
    let (r, robots) = parse_input(input).expect("valid input");
    assert!(r.is_empty());

    let g = Grid { x: 101, y: 103 };

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

// How does a christmas tree look like:
//     *
//    ***
//   *   *
//  *     *
// ***   ***
//  *     *
// *       *
//
// So general logic seems to be: for every row, at most 2 runs of robots
// DOES NOT WORK
fn is_suspicious_shape1(g: &Grid, pos: &HashSet<IVec2>) -> bool {
    for y in 0..(g.y as i32) {
        let cnt = (0..(g.x as i32))
            .map(|x| (x - 1, x))
            .map(|(x1, x2)| {
                (
                    pos.contains(&IVec2::new(x1, y)),
                    pos.contains(&IVec2::new(x2, y)),
                )
            })
            .filter(|(p1, p2)| !*p1 && *p2)
            .count();

        if cnt > 2 {
            return false;
        }
    }
    true
}

/// let all the robots BE CONNECTED (N/E/S/W)
/// DOES NOT WORK
fn is_suspicious_shape2(_: &Grid, pos: &HashSet<IVec2>) -> bool {
    let mut to_check = VecDeque::new();
    to_check.push_back(*pos.iter().next().expect("non-empty set"));

    let mut connected = HashSet::new();

    while let Some(value) = to_check.pop_front() {
        connected.insert(value);
        for x in -1..=1 {
            for y in -1..=1 {
                let other = value + IVec2::new(x, y);
                if !connected.contains(&other) && pos.contains(&other) {
                    to_check.push_back(other);
                }
            }
        }
    }

    connected.len() == pos.len()
}

// this works for SOME at 7138 ...
fn is_suspicious_shape(_: &Grid, pos: &HashSet<IVec2>) -> bool {
    // find the largest connected line and filter based on that ...
    let mut connected = HashSet::new();
    while &connected != pos {
        let mut to_check = VecDeque::new();
        to_check.push_back(*pos.difference(&connected).next().expect("non-empty set"));

        let mut cnt = 1;
        while let Some(value) = to_check.pop_front() {
            connected.insert(value);
            cnt += 1;
            for (x, y) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
                let other = value + IVec2::new(x, y);
                if !connected.contains(&other) && pos.contains(&other) {
                    to_check.push_back(other);
                }
            }
        }
        if cnt > 60 {
            return true;
        }
    }
    false
}

pub fn part2(input: &str) -> usize {
    let (r, robots) = parse_input(input).expect("valid input");
    assert!(r.is_empty());

    let g = Grid { x: 101, y: 103 };

    let x = (0..(g.x * g.y))
        .into_par_iter()
        .map(|sc| {
            let pos = robots.iter().map(|r| g.move_robot(r, sc)).collect();
            (sc, is_suspicious_shape(&g, &pos))
        })
        .find_first(|x| x.1)
        .expect("has something");
    // let pos = robots.iter().map(|r| g.move_robot(r, x.0)).collect();
    // g.display_robots(&pos, x.0);

    x.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_move() {
        let g = Grid { x: 11, y: 7 };
        assert_eq!(
            g.move_robot(
                &Robot {
                    position: IVec2::new(2, 4),
                    velocity: IVec2::new(2, -3)
                },
                1
            ),
            IVec2::new(4, 1)
        );

        assert_eq!(
            g.move_robot(
                &Robot {
                    position: IVec2::new(2, 4),
                    velocity: IVec2::new(2, -3)
                },
                2
            ),
            IVec2::new(6, 5)
        );
    }
}
