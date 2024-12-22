use std::{
    collections::{HashMap, HashSet},
    default,
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
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

#[derive(Debug)]
struct Input {
    inputs: Vec<String>,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, inputs) = separated_list1(
        line_ending,
        is_a("0123456789A").map(|s: &str| s.to_string()),
    )
    .terminated(many0(line_ending))
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(Input { inputs })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

#[derive(Default)]
struct KeyPad {
    coord: HashMap<char, IVec2>,
    gap: IVec2, // where is the gap in the keyboard

    // cache
    short_paths_cache: HashMap<String, HashSet<String>>,
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
            ..Default::default()
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
            ..Default::default()
        }
    }

    /// all the shortest paths (max 2 of them) from `from` to `to`
    /// making sure we do not go over the gap
    fn shortest_moves(&self, from: IVec2, to: IVec2) -> Vec<String> {
        let x_moves = if to.x > from.x {
            vec!['>'; (to.x - from.x) as usize]
        } else {
            vec!['<'; (from.x - to.x) as usize]
        };

        let y_moves = if from.y > to.y {
            vec!['^'; (from.y - to.y) as usize]
        } else {
            vec!['v'; (to.y - from.y) as usize]
        };

        let mut result = Vec::new();

        if IVec2::new(from.x, to.y) != self.gap {
            let mut s = String::with_capacity(x_moves.len() + y_moves.len());
            s.extend(y_moves.iter());
            s.extend(x_moves.iter());
            result.push(s);
        }

        if IVec2::new(to.x, from.y) != self.gap {
            let mut s = String::with_capacity(x_moves.len() + y_moves.len());
            s.extend(x_moves.iter());
            s.extend(y_moves.iter());
            result.push(s);
        }

        result
    }

    fn short_key_paths(&mut self, target: &str) -> HashSet<String> {
        let mut pos = *self.coord.get(&'A').expect("A has a position");

        let mut moves = HashSet::new();
        moves.insert("".to_string());

        match self.short_paths_cache.get(target) {
            Some(value) => return value.clone(),
            _ => {}
        };

        for c in target.chars() {
            let dest = *self
                .coord
                .get(&c)
                .unwrap_or_else(|| panic!("{} MUST BE A valid destination", c));

            // extend every single move with what we can
            moves = moves
                .iter()
                .flat_map(|m| {
                    self.shortest_moves(pos, dest)
                        .iter()
                        .map(|suffix| {
                            let mut prefix = m.to_string();
                            prefix.push_str(suffix);
                            prefix.push('A');
                            prefix
                        })
                        .collect::<Vec<String>>()
                })
                .collect();

            // keep only the shortest paths, otherwise there is no point
            let minlen = moves.iter().map(|m| m.len()).min().expect("has moves");
            moves = moves
                .iter()
                .filter(|m| m.len() == minlen)
                .map(|m| m.to_owned())
                .collect();

            pos = dest;
        }

        self.short_paths_cache
            .insert(target.to_string(), moves.clone());

        moves
    }
}

fn code_number(target: &str) -> usize {
    assert_eq!(target.chars().count(), 4);
    assert_eq!(target.chars().last(), Some('A'));
    target.split_at(3).0.parse::<usize>().expect("valid number")
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut keypad = KeyPad::new_button_pad();
    let mut arrow_pad = KeyPad::new_arrow_pad();

    Ok(input
        .inputs
        .iter()
        .map(|code| {
            let number = code_number(code);

            let mut c = keypad.short_key_paths(code);
            tracing::info!("DEBUG: {}", code);
            for _ in 0..2 {
                tracing::info!("    NEXT: {:?}", c.iter().take(3).collect::<Vec<_>>());
                c = c
                    .iter()
                    .flat_map(|x| arrow_pad.short_key_paths(x))
                    .collect();
            }
            tracing::info!("    NEXT: {:?}", c.iter().take(3).collect::<Vec<_>>());

            let mincode = c.iter().map(|s| s.len()).min().expect("Has min");

            tracing::info!("{}: CODE {} and mincode {}", code, number, mincode);
            number * mincode
        })
        .sum())
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut keypad = KeyPad::new_button_pad();
    let mut arrow_pad = KeyPad::new_arrow_pad();

    Ok(input
        .inputs
        .iter()
        .map(|code| {
            let number = code_number(code);

            let mut c = keypad.short_key_paths(code);
            tracing::info!("DEBUG: {}", code);
            for _ in 0..25 {
                tracing::info!("    NEXT: {:?}", c.iter().take(3).collect::<Vec<_>>());
                c = c
                    .iter()
                    .flat_map(|x| arrow_pad.short_key_paths(x))
                    .collect();
            }
            tracing::info!("    NEXT: {:?}", c.iter().take(3).collect::<Vec<_>>());

            let mincode = c.iter().map(|s| s.len()).min().expect("Has min");

            tracing::info!("{}: CODE {} and mincode {}", code, number, mincode);
            number * mincode
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
    fn test_part1() {
        init_tests();
        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            126384
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
