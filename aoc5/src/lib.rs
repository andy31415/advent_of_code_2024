use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Ordering {
    before: u32,
    after: u32,
}

#[derive(Debug)]
struct Input {
    ordering: Vec<Ordering>,
    lines: Vec<Vec<u32>>,
}

mod parse {
    use nom::{
        bytes::complete::tag,
        character::complete::{self, newline},
        multi::{count, many0, separated_list0, separated_list1},
        sequence::separated_pair,
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    fn ordering(s: &str) -> IResult<&str, Vec<super::Ordering>> {
        separated_list0(
            newline,
            separated_pair(complete::u32, tag("|"), complete::u32)
                .map(|(before, after)| super::Ordering { before, after }),
        )
        .parse(s)
    }

    pub(crate) fn parse(s: &str) -> IResult<&str, super::Input> {
        separated_pair(
            ordering,
            count(newline, 2),
            separated_list0(newline, separated_list1(tag(","), complete::u32)),
        )
        .terminated(many0(newline))
        .map(|(ordering, lines)| super::Input { ordering, lines })
        .parse(s)
    }
}

trait MidValued {
    fn mid_value(&self) -> u32;
}

impl MidValued for Vec<u32> {
    #[tracing::instrument]
    fn mid_value(&self) -> u32 {
        assert!(self.len() % 2 == 1);
        self[self.len() >> 1]
    }
}

#[tracing::instrument]
pub fn is_priority_respected(data: &[u32], before_to_after: &HashMap<u32, HashSet<u32>>) -> bool {
    let mut seen = HashSet::new();
    let empty_set = HashSet::new();

    for v in data.iter() {
        if seen
            .intersection(before_to_after.get(v).unwrap_or(&empty_set))
            .count()
            > 0
        {
            return false;
        }

        seen.insert(*v);
    }

    true
}

fn fix_priority(data: &[u32], before_to_after: &HashMap<u32, HashSet<u32>>) -> Vec<u32> {
    // For every entry, place the entry just before anything that already is supposed to be in
    // front of it
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let empty_set = HashSet::new();

    for v in data {
        let must_be_before = before_to_after.get(v).unwrap_or(&empty_set);

        let insert_pos = result
            .iter()
            .enumerate()
            .find(|(_, value)| must_be_before.contains(value));

        match insert_pos {
            Some((idx, _)) => result.insert(idx, *v),
            None => result.push(*v),
        }

        seen.insert(*v);
    }

    result
}

pub fn part1(input: &str) -> u32 {
    let (r, input) = parse::parse(input).expect("valid input");
    assert!(r.is_empty());

    let mut before_to_after = HashMap::new();
    for o in input.ordering {
        before_to_after
            .entry(o.before)
            .or_insert_with(HashSet::new)
            .insert(o.after);
    }

    input
        .lines
        .iter()
        .filter(|v| is_priority_respected(v, &before_to_after))
        .map(MidValued::mid_value)
        .sum()
}

pub fn part2(input: &str) -> u32 {
    let (r, input) = parse::parse(input).expect("valid input");
    assert!(r.is_empty());

    let mut before_to_after = HashMap::new();
    for o in input.ordering {
        before_to_after
            .entry(o.before)
            .or_insert_with(HashSet::new)
            .insert(o.after);
    }

    input
        .lines
        .iter()
        .filter(|v| !is_priority_respected(v, &before_to_after))
        .map(|v| fix_priority(v, &before_to_after))
        .map(|v| MidValued::mid_value(&v))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mid_value() {
        assert_eq!(vec![100].mid_value(), 100);
        assert_eq!(vec![1, 2, 3].mid_value(), 2);
        assert_eq!(vec![1, 2, 3, 4, 5].mid_value(), 3);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 143);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 123);
    }
}
