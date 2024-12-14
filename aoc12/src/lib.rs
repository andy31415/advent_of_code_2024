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

pub fn part1(input: &str) -> usize {
    let (r, m) = map_parse::Map::<Plant>::parse(input).expect("Valid input");
    assert!(r.is_empty());
    // TODO: implement
    0
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
        assert_eq!(part1(include_str!("../example.txt")), 1930);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
