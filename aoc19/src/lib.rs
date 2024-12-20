use std::collections::HashSet;

use nom::{
    bytes::complete::{is_a, tag},
    character::complete::{line_ending, satisfy},
    multi::{many0, many1, separated_list1},
    sequence::{self, tuple},
    IResult, Parser as _,
};
use nom_supreme::ParserExt;
use rayon::prelude::*;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

#[derive(Debug)]
struct Stripe {
    pattern: String,
}

#[derive(Debug)]
struct Input {
    available: Vec<Stripe>,
    required: Vec<Stripe>,
}

fn parse_stripe(s: &str) -> IResult<&str, Stripe> {
    is_a("wubrg")
        .map(|p: &str| Stripe {
            pattern: p.to_string(),
        })
        .parse(s)
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, input) = tuple((
        separated_list1(tag(", "), parse_stripe).terminated(many0(line_ending)),
        separated_list1(line_ending, parse_stripe).terminated(many0(line_ending)),
    ))
    .map(|(available, required)| Input {
        available,
        required,
    })
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(input)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

#[derive(Default)]
struct GoalCache {
    choices: Vec<Stripe>,
    possible: HashSet<String>,
    impossible: HashSet<String>,
}

impl GoalCache {
    #[tracing::instrument(ret, skip(self))]
    pub fn can_build(&mut self, goal: &str) -> bool {
        if goal.is_empty() {
            return true;
        }
        if self.possible.contains(goal) {
            return true;
        }
        if self.impossible.contains(goal) {
            return false;
        }

        let tails = self
            .choices
            .iter()
            .filter(|a| goal.starts_with(&a.pattern))
            .map(|a| goal.split_at(a.pattern.len()).1)
            .collect::<Vec<_>>();

        for tail in tails {
            if self.can_build(tail) {
                self.possible.insert(goal.to_string());
                return true;
            }
        }

        self.impossible.insert(goal.to_string());
        false
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut cache = GoalCache {
        choices: input.available,
        ..Default::default()
    };

    let mut result = 0;
    for (idx, v) in input.required.iter().enumerate() {
        if cache.can_build(&v.pattern) {
            result += 1;
        }
    }

    Ok(result)
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

    #[test_log::test]
    fn test_part1() {
        init_tests();
        assert_eq!(part1(include_str!("../example.txt")).expect("success"), 6);
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
