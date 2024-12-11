use std::collections::HashMap;

use nom::{
    character::complete::{self, line_ending, space0},
    multi::{many0, many1},
    IResult, Parser,
};
use nom_supreme::ParserExt;

#[derive(Debug, Default)]
struct BlinkCache {
    cache: HashMap<(usize, usize), usize>,
}

impl BlinkCache {
    pub fn multiply(&mut self, stone: usize, blinks: usize) -> usize {
        if let Some(value) = self.cache.get(&(stone, blinks)) {
            return *value;
        }

        if blinks == 0 {
            return 1;
        }

        // rule 1: 0 replaces with 1
        if stone == 0 {
            // rule
            let value = self.multiply(1, blinks - 1);
            self.cache.insert((stone, blinks), value);
            return value;
        }

        // two digits: 10-100
        // four digits: 1000 - 10000
        // six digits: 100000 - 1000000
        // and so on
        // always [low; high)
        let mut low = 10;
        let mut high = 100;
        let mut half = 10;

        while high < stone {
            half *= 10;
            low *= 100;
            high *= 100;
        }

        if stone >= low {
            // we KNOW high < stone
            // split in half
            let value =
                self.multiply(stone % half, blinks - 1) + self.multiply(stone / half, blinks - 1);
            self.cache.insert((stone, blinks), value);
            return value;
        }

        let value = self.multiply(stone * 2024, blinks - 1);
        self.cache.insert((stone, blinks), value);
        value
    }
}

pub fn parse_input(s: &str) -> IResult<&str, Vec<usize>> {
    many1(complete::u32.map(|v| v as usize).terminated(space0))
        .terminated(many0(line_ending))
        .parse(s)
}

pub fn part1(input: &str) -> usize {
    let (r, v) = parse_input(input).expect("valid input");
    assert!(r.is_empty());

    let mut cache = BlinkCache::default();

    v.iter().map(|v| cache.multiply(*v, 25)).sum()
}

pub fn part2(input: &str) -> usize {
    let (r, v) = parse_input(input).expect("valid input");
    assert!(r.is_empty());

    let mut cache = BlinkCache::default();

    v.iter().map(|v| cache.multiply(*v, 75)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_stepwise() {
        let mut cache = BlinkCache::default();
        assert_eq!(cache.multiply(1, 0), 1);
        assert_eq!(cache.multiply(0, 0), 1);
        assert_eq!(cache.multiply(100, 0), 1);

        assert_eq!(cache.multiply(0, 1), 1);
        assert_eq!(cache.multiply(1, 1), 1);
        assert_eq!(cache.multiply(9, 1), 1);
        assert_eq!(cache.multiply(10, 1), 2);
        assert_eq!(cache.multiply(22, 1), 2);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 55312);
    }

    #[test]
    fn test_part2() {
        // this was not given in the test, but we added it here based on passing test anyway
        assert_eq!(part2(include_str!("../example.txt")), 65601038650482);
    }
}
