use std::collections::HashSet;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Equation {
    target: u64,
    parts: Vec<u64>,
}

impl Equation {
    fn solvable_by_add_mult(&self) -> bool {
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
                    let mut new_targets = HashSet::new();

                    for x in h.iter() {
                        if x >= item {
                            new_targets.insert(x - item);
                        }
                        if x % item == 0 {
                            new_targets.insert(x / item);
                        }
                    }
                    new_targets
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

pub fn part1(input: &str) -> u64 {
    let (r, equations) = parse::equations(input).expect("valid input");
    assert!(r.is_empty());

    equations
        .iter()
        .filter(|e| Equation::solvable_by_add_mult(e))
        .map(|e| e.target)
        .sum()
}

pub fn part2(input: &str) -> u64 {
    // TODO: implement
    0
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
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
