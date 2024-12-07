use std::collections::HashSet;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Equation {
    target: u64,
    parts: Vec<u64>,
}

impl Equation {
    fn solvable_by<C: Fn(u64, u64) -> Option<u64>>(&self, conversions: &[&C]) -> bool {
        let mut choices = HashSet::new();

        choices.insert(self.target);

        self.parts
            .iter()
            .skip(1)
            .rev()
            .fold(
                {
                    let mut h = HashSet::new();
                    h.insert(self.target);
                    h
                },
                |h, item| {
                    h.iter()
                        .flat_map(|x| conversions.iter().filter_map(|c| c(*x, *item)))
                        .collect()
                },
            )
            .contains(self.parts.first().expect("has at least one item"))
    }
}

mod parse {
    use super::Equation;
    use nom::{
        bytes::complete::tag,
        character::complete::{self, line_ending, space0, space1},
        multi::{many0, separated_list1},
        sequence::separated_pair,
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    pub(crate) fn equation(s: &str) -> IResult<&str, Equation> {
        separated_pair(
            complete::u64,
            tag(":"),
            separated_list1(space1, complete::u64).preceded_by(space0),
        )
        .map(|(target, parts)| Equation { target, parts })
        .parse(s)
    }

    pub(crate) fn equations(s: &str) -> IResult<&str, Vec<Equation>> {
        separated_list1(line_ending, equation)
            .terminated(many0(line_ending))
            .parse(s)
    }
}

fn substract(target: u64, x: u64) -> Option<u64> {
    if target < x {
        return None;
    }
    Some(target - x)
}

fn divide(target: u64, x: u64) -> Option<u64> {
    if target % x != 0 {
        return None;
    }
    Some(target / x)
}

fn remove_suffix(target: u64, x: u64) -> Option<u64> {
    let mut ts = target.to_string();
    let xs = x.to_string();

    if !ts.ends_with(&xs) || ts.len() <= xs.len() {
        return None;
    }

    ts.truncate(ts.len() - xs.len());
    Some(ts.parse().expect("valid number"))
}

pub fn part1(input: &str) -> u64 {
    let (r, equations) = parse::equations(input).expect("valid input");
    assert!(r.is_empty());

    equations
        .iter()
        .filter(|e| {
            e.solvable_by(&[
                &(substract as fn(u64, u64) -> Option<u64>),
                &(divide as fn(u64, u64) -> Option<u64>),
            ])
        })
        .map(|e| e.target)
        .sum()
}

pub fn part2(input: &str) -> u64 {
    let (r, equations) = parse::equations(input).expect("valid input");
    assert!(r.is_empty());

    equations
        .iter()
        .filter(|e| {
            e.solvable_by(&[
                &(substract as fn(u64, u64) -> Option<u64>),
                &(divide as fn(u64, u64) -> Option<u64>),
                &(remove_suffix as fn(u64, u64) -> Option<u64>),
            ])
        })
        .map(|e| e.target)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 3749);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 11387);
    }
}
