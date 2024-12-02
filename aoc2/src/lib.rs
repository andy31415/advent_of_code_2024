use itertools::zip;
use parse::input;

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

/// Safe means:
///    - strictly increasing/decreasing
///    - at least one, at most 3
fn is_safe(vec: &Vec<u32>) -> bool {
    let mut up = false;
    let mut down = false;
    for (a, b) in zip(vec.iter(), vec.iter().skip(1)) {
        if b.abs_diff(*a) > 3 {
            return false;
        }

        match a.cmp(b) {
            std::cmp::Ordering::Less => down = true,
            std::cmp::Ordering::Greater => up = true,
            std::cmp::Ordering::Equal => return false,
        }

        if up && down {
            return false;
        }
    }

    true
}

pub fn part1(s: &str) -> usize {
    let (r, data) = input(s).expect("good input");
    assert!(r.is_empty());

    let mut cnt = 0;
    for n in data.levels.into_iter().filter(is_safe) {
        cnt += 1
    }
    cnt
}

pub fn part2(s: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {

    use super::parse::*;
    use super::*;

    #[test]
    fn test_safe() {
        assert!(is_safe(&vec![7u32, 6u32, 4u32, 2u32, 1u32]));
        assert!(!is_safe(&vec![7u32, 7u32, 4u32, 2u32, 1u32]));
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
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 2);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
