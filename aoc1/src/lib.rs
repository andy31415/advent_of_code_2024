use std::iter::zip;


#[derive(Debug, PartialEq)]
struct Input {
    v1: Vec<u32>,
    v2: Vec<u32>,
}

mod parse {

use nom::{character::complete::{digit1, line_ending, multispace1, u32 as parse_u32}, multi::{many0, separated_list1}, sequence::separated_pair, IResult, Parser};
use nom_supreme::ParserExt;
use super::Input;

pub(crate) fn pair(input: &str) -> IResult<&str, (u32, u32)> {
    separated_pair(parse_u32, multispace1 , parse_u32)
        .parse(input)
}

pub fn input(input: &str) -> IResult<&str, Input> {
   separated_list1(line_ending, pair)
       .terminated(many0(line_ending))
       .map(|pairs| {
           Input{
               v1: pairs.iter().map(|v| {v.0}).collect(),
               v2: pairs.iter().map(|v| {v.1}).collect(),
           }
       })
       .parse(input)
}

}


pub fn part1(input: &str) -> u32 {
    let (r, mut d) = parse::input(input).expect("Valid input");
    assert_eq!(r, "");

    d.v1.sort();
    d.v2.sort();

    zip(d.v1, d.v2).fold(0u32, |v, p| {
        v + p.1.abs_diff(p.0)
    })
}

pub fn part2(_input: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse::pair("1 2").expect("valid").1, (1, 2));
        assert_eq!(parse::pair("123   100").expect("valid").1, (123, 100));
        assert_eq!(parse::pair("4   3").expect("valid").1, (4, 3));
    }

    #[test]
    fn test_parse_input() {
        assert_eq!(parse::input("1 2\n3 4").expect("valid").1, Input {
            v1: vec![1u32, 3u32],
            v2: vec![2u32, 4u32],
        });
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 11);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
