use nom::{
    character::complete::{self, line_ending, space0},
    multi::{many0, many1},
    IResult, Parser,
};
use nom_supreme::ParserExt;

pub fn parse_input(s: &str) -> IResult<&str, Vec<u32>> {
    many1(complete::u32.terminated(space0))
        .terminated(many0(line_ending))
        .parse(s)
}

pub fn part1(input: &str) -> usize {
    let (r, v) = parse_input(input).expect("valid input");
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
        assert_eq!(part1(include_str!("../example.txt")), 55312);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
