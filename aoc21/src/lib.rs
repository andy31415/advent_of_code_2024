use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use glam::IVec2;
use nom::{
    bytes::complete::is_a,
    character::complete::line_ending,
    multi::{many0, separated_list1},
    Parser,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum ProcessingError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),

    #[error("No coordinates for: {0:?} (invalid character?)")]
    InvalidCharacter(char),
}

#[derive(Debug)]
struct Input {
    inputs: Vec<String>,
}

fn parse_input(s: &str) -> Result<Input, ProcessingError> {
    let (rest, inputs) = separated_list1(
        line_ending,
        is_a("0123456789A").map(|s: &str| s.to_string()),
    )
    .terminated(many0(line_ending))
    .parse(s)?;

    if !rest.is_empty() {
        return Err(ProcessingError::UnparsedData(rest.into()));
    }

    Ok(Input { inputs })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for ProcessingError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        ProcessingError::NomError(value.map_input(|i| i.into()))
    }
}

#[derive(Default)]
struct KeyPad {
    coord: HashMap<char, IVec2>,
    gap: IVec2, // where is the gap in the keyboard
}

impl KeyPad {
    fn new_button_pad() -> Self {
        let mut coord = HashMap::new();

        coord.insert('7', (0, 0).into());
        coord.insert('8', (1, 0).into());
        coord.insert('9', (2, 0).into());
        coord.insert('4', (0, 1).into());
        coord.insert('5', (1, 1).into());
        coord.insert('6', (2, 1).into());
        coord.insert('1', (0, 2).into());
        coord.insert('2', (1, 2).into());
        coord.insert('3', (2, 2).into());
        coord.insert('0', (1, 3).into());
        coord.insert('A', (2, 3).into());

        Self {
            coord,
            gap: IVec2::new(0, 3),
        }
    }

    fn new_arrow_pad() -> Self {
        let mut coord = HashMap::new();

        coord.insert('^', (1, 0).into());
        coord.insert('A', (2, 0).into());
        coord.insert('<', (0, 1).into());
        coord.insert('v', (1, 1).into());
        coord.insert('>', (2, 1).into());

        Self {
            coord,
            gap: IVec2::new(0, 0),
        }
    }

    fn all_shortest_paths(&self, from: char, to: char) -> Result<HashSet<String>, ProcessingError> {
        let from = match self.coord.get(&from) {
            Some(value) => value,
            None => return Err(ProcessingError::InvalidCharacter(from)),
        };

        let to = match self.coord.get(&to) {
            Some(value) => value,
            None => return Err(ProcessingError::InvalidCharacter(to)),
        };

        let mut result = HashSet::new();

        // now find all from/to positions. These are at most 3x4 paths, so hopefully not too many
        let mut to_check = VecDeque::new();
        to_check.push_back((*from, "".to_string()));

        while let Some((pos, path)) = to_check.pop_front() {
            if pos == self.gap {
                // not allowed to get there
                continue;
            }
            if pos == *to {
                result.insert(path);
                continue;
            }

            // decide how to move
            if to.x > pos.x {
                to_check.push_back((pos + IVec2::new(1, 0), path.clone() + ">"));
            }
            if to.x < pos.x {
                to_check.push_back((pos + IVec2::new(-1, 0), path.clone() + "<"));
            }
            if to.y > pos.y {
                to_check.push_back((pos + IVec2::new(0, 1), path.clone() + "v"));
            }
            if to.y < pos.y {
                to_check.push_back((pos + IVec2::new(0, -1), path.clone() + "^"));
            }
        }

        Ok(result)
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
struct CacheTarget {
    from: char,
    to: char,
    depth: usize,
}

#[tracing::instrument(ret, skip(cache, pads))]
fn short_path_count(code: &str, pads: &[KeyPad], cache: &mut HashMap<CacheTarget, usize>) -> usize {
    let mut current_pos = 'A'; //we start here
    let mut cnt = 0;

    let (pad, rest) = match pads.split_first() {
        Some((a, b)) => (a, b),
        None => return code.len(),
    };

    for dest in code.chars() {
        // try all possible paths and figure out how to get there
        tracing::info!("GOING FROM {} to {}", current_pos, dest);

        let target = CacheTarget {
            from: current_pos,
            to: dest,
            depth: rest.len(),
        };
        if let Some(value) = cache.get(&target) {
            tracing::info!("CACHE HIT!");
            cnt += value;
        } else {
            let compute = pad
                .all_shortest_paths(current_pos, dest)
                .expect("has paths")
                .iter()
                .map(|path| short_path_count(&(path.clone() + "A"), rest, cache))
                .min()
                .expect("Has some path");

            cache.insert(target, compute);
            cnt += compute;
        }

        current_pos = dest;
    }
    cnt
}

fn code_number(target: &str) -> usize {
    assert_eq!(target.chars().count(), 4);
    assert_eq!(target.chars().last(), Some('A'));
    target.split_at(3).0.parse::<usize>().expect("valid number")
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut pads = Vec::new();

    pads.push(KeyPad::new_button_pad());
    pads.push(KeyPad::new_arrow_pad());
    pads.push(KeyPad::new_arrow_pad());

    Ok(input
        .inputs
        .iter()
        .map(|code| {
            let number = code_number(code);
            let cnt = short_path_count(code, &pads, &mut HashMap::new());
            number * cnt
        })
        .sum())
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut pads = Vec::new();

    pads.push(KeyPad::new_button_pad());
    for _ in 0..25 {
        pads.push(KeyPad::new_arrow_pad());
    }

    Ok(input
        .inputs
        .iter()
        .map(|code| {
            let number = code_number(code);
            let cnt = short_path_count(code, &pads, &mut HashMap::new());
            number * cnt
        })
        .sum())
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
    fn test_short_path() {
        let arrow_pad = KeyPad::new_arrow_pad();

        assert_eq!(
            arrow_pad.all_shortest_paths('A', '<').expect("ok"),
            ["<v<".to_string(), "v<<".to_string(),].into()
        );

        let key_pad = KeyPad::new_button_pad();
        assert_eq!(
            key_pad.all_shortest_paths('4', '3').expect("ok"),
            [">>v".to_string(), ">v>".to_string(), "v>>".to_string(),].into()
        );

        assert_eq!(
            key_pad.all_shortest_paths('4', 'A').expect("ok"),
            [
                ">>vv".to_string(),
                ">v>v".to_string(),
                "v>>v".to_string(),
                "v>v>".to_string(),
                ">vv>".to_string(),
            ]
            .into()
        );
    }

    #[test_log::test]
    fn test_part1() {
        init_tests();

        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            126384
        );
    }
}
