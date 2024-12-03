use std::iter::zip;

use itertools::enumerate;
use parse::input;
use tracing::{event, instrument, Level};

#[derive(Debug, PartialEq)]
struct Input {
    levels: Vec<Vec<u32>>,
}

mod parse {
    use nom::{
        character::complete::{newline, space1, u32 as parse_u32},
        multi::{many0, many1, separated_list1},
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    pub(crate) fn level(s: &str) -> IResult<&str, Vec<u32>> {
        separated_list1(space1, parse_u32).parse(s)
    }

    pub(crate) fn input(s: &str) -> IResult<&str, super::Input> {
        separated_list1(many1(newline), level)
            .terminated(many0(newline))
            .map(|levels| super::Input { levels })
            .parse(s)
    }
}

#[derive(PartialEq, Debug)]
enum Safety {
    Up,
    Down,
    OutOfRange,
}

fn is_safe(a: u32, b: u32) -> Safety {
    if b.abs_diff(a) > 3 {
        return Safety::OutOfRange;
    }

    match a.cmp(&b) {
        std::cmp::Ordering::Less => Safety::Up,
        std::cmp::Ordering::Greater => Safety::Down,
        std::cmp::Ordering::Equal => Safety::OutOfRange,
    }
}

/// Safe means:
///    - strictly increasing/decreasing
///    - at least one, at most 3
fn is_safe_level(vec: &[u32]) -> bool {
    let mut safety = None;

    for (a, b) in zip(vec.iter(), vec.iter().skip(1)) {
        let cur = is_safe(*a, *b);

        match safety {
            None => safety = Some(cur),
            Some(value) if value != cur => return false,
            _ => {}
        }
    }

    true
}

#[instrument(ret)]
fn safe_level_if_remove(vec: &[u32], idx: usize) -> bool {
    if idx >= vec.len() {
        return false;
    }

    let mut oth: Vec<u32> = vec.into();
    oth.remove(idx);

    is_safe_level(&oth)
}

/// Figure out which positions something unsafe resides on
#[instrument(ret)]
fn is_safe_by_removal(vec: &[u32]) -> bool {
    let states = zip(vec.iter(), vec.iter().skip(1))
        .map(|(a, b)| is_safe(*a, *b))
        .collect::<Vec<_>>();

    // at this point, invalid locations are left/right when there is a greater or a single up/down
    // difference

    let ups = enumerate(states.iter())
        .filter(|(_, a)| **a == Safety::Up)
        .collect::<Vec<_>>();
    let downs = enumerate(states.iter())
        .filter(|(_, a)| **a == Safety::Down)
        .collect::<Vec<_>>();
    let ranges = enumerate(states.iter())
        .filter(|(_, a)| **a == Safety::OutOfRange)
        .collect::<Vec<_>>();

    match ranges.len() {
        0 => { /* ok, all in range */ }
        1 => {
            let (p1, _) = ranges.first().expect("has 1 element");
            event!(Level::INFO, "Out of rangge at index {}", *p1);
            return safe_level_if_remove(vec, *p1) || safe_level_if_remove(vec, *p1 + 1);
        }
        2 => {
            let (p1, _) = ranges.first().expect("has 2 elements");
            let (p2, _) = ranges.get(1).expect("has 2 elements");

            event!(Level::INFO, "Out of rangge at index {} and {}", *p1, *p2);

            if *p2 != *p1 + 1 {
                return false;
            }

            return safe_level_if_remove(vec, *p2);
        }
        _ => return false,
    }

    // we need to try to remove ups or downs
    if ups.len() == 1 {
        let (p1, _) = ups.first().expect("has 1 element");
        event!(Level::INFO, "Single UP at  index {}", *p1);
        return safe_level_if_remove(vec, *p1) || safe_level_if_remove(vec, *p1 + 1);
    }

    if downs.len() == 1 {
        let (p1, _) = downs.first().expect("has 1 element");
        event!(Level::INFO, "Single DOWN at  index {}", *p1);
        return safe_level_if_remove(vec, *p1) || safe_level_if_remove(vec, *p1 + 1);
    }

    is_safe_level(vec)
}

pub fn part1(s: &str) -> usize {
    let (r, data) = input(s).expect("good input");
    assert!(r.is_empty());

    data.levels.into_iter().filter(|x| is_safe_level(x)).count()
}

pub fn part2(s: &str) -> usize {
    let (r, data) = input(s).expect("good input");
    assert!(r.is_empty());

    data.levels
        .into_iter()
        .filter(|x| is_safe_by_removal(x))
        .count()
}

#[cfg(test)]
mod tests {

    use super::parse::*;
    use super::*;

    #[test]
    fn test_safe() {
        assert!(is_safe_level(&[7u32, 6u32, 4u32, 2u32, 1u32]));
        assert!(!is_safe_level(&[7u32, 7u32, 4u32, 2u32, 1u32]));
    }

    #[test]
    fn test_parse_level() {
        assert_eq!(level("1 2 3").expect("valid").1, vec![1u32, 2u32, 3u32])
    }

    #[test]
    fn test_parse_input() {
        assert_eq!(
            input("1 2 3\n4 5").expect("valid").1,
            Input {
                levels: vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32],]
            }
        )
    }

    #[test]
    fn test_safe_level_if_remove() {
        let v = vec![1u32, 2u32, 3u32, 10u32, 4u32];

        assert!(!safe_level_if_remove(&v, 0));
        assert!(!safe_level_if_remove(&v, 1));
        assert!(!safe_level_if_remove(&v, 2));
        assert!(safe_level_if_remove(&v, 3));
        assert!(!safe_level_if_remove(&v, 4));
        assert!(!safe_level_if_remove(&v, 5));

        let v = vec![10u32, 2u32, 3u32, 10u32, 4u32];
        assert!(!safe_level_if_remove(&v, 0));
        assert!(!safe_level_if_remove(&v, 3));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 2);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 4);
    }
}
