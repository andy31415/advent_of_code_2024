use std::num::ParseIntError;

use regex::Captures;

#[derive(PartialEq, Eq, Debug)]
struct Multiplication {
    left: usize,
    right: usize,
}

impl Multiplication {
    fn value(&self) -> usize {
        self.left * self.right
    }
}

impl TryFrom<Captures<'_>> for Multiplication {
    type Error = ParseIntError;

    fn try_from(m: Captures) -> Result<Self, Self::Error> {
        Ok(Multiplication {
            left: m.name("left").expect("Has a match").as_str().parse()?,
            right: m.name("right").expect("Has a match").as_str().parse()?,
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Input {
    items: Vec<Multiplication>,
}

enum Instruction {
    Do,
    Dont,
    Mul(Multiplication),
}

impl TryFrom<Captures<'_>> for Instruction {
    type Error = ParseIntError;

    fn try_from(m: Captures) -> Result<Self, Self::Error> {
        Ok(match m.get(0).expect("0 always exists").as_str() {
            "do()" => Instruction::Do,
            "don't()" => Instruction::Dont,
            _ => Instruction::Mul(m.try_into()?),
        })
    }
}

struct Input2 {
    items: Vec<Instruction>,
}

mod parse {

    use super::{Input, Input2};
    use regex::Regex;

    pub(crate) fn input(s: &str) -> Input {
        let expr = Regex::new(r"mul\((?<left>\d+),(?<right>\d+)\)").expect("Valid expression");

        Input {
            items: expr
                .captures_iter(s)
                .map(|m| m.try_into().expect("valid"))
                .collect(),
        }
    }

    pub(crate) fn input2(s: &str) -> Input2 {
        let expr = Regex::new(r"mul\((?<left>\d+),(?<right>\d+)\)|do\(\)|don't\(\)")
            .expect("Valid expression");

        Input2 {
            items: expr
                .captures_iter(s)
                .map(|m| m.try_into().expect("valid"))
                .collect(),
        }
    }
}

pub fn part1(s: &str) -> usize {
    parse::input(s)
        .items
        .iter()
        .map(Multiplication::value)
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

pub fn part2(s: &str) -> usize {
    let mut result = 0;
    let mut on = true;

    for item in parse::input2(s).items.iter() {
        match item {
            Instruction::Do => on = true,
            Instruction::Dont => on = false,
            Instruction::Mul(m) => {
                if on {
                    result += m.value();
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::parse::*;
    use super::*;

    #[test]
    fn test_input_parse() {
        assert_eq!(input(""), Input { items: vec![] });
        assert_eq!(input("test"), Input { items: vec![] });
        assert_eq!(input("mul(1,2"), Input { items: vec![] });
        assert_eq!(
            input("mul(1,2)"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("mul(1,2)foo"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("foomul(1,2)bar"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("foomul(1,2)barmul(3,4)"),
            Input {
                items: vec![
                    Multiplication { left: 1, right: 2 },
                    Multiplication { left: 3, right: 4 },
                ]
            }
        );
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 161);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 48);
    }
}
