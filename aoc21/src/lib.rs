use std::collections::HashMap;

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

    fn short_key_path(&self, target: &str) -> String {
        let mut pos = *self.coord.get(&'A').expect("A has a position");

        let mut moves = String::new();

        for c in target.chars() {
            let dest = *self
                .coord
                .get(&c)
                .unwrap_or_else(|| panic!("{} MUST BE A valid destination", c));

            let x_moves = if dest.x > pos.x {
                vec!['>'; (dest.x - pos.x) as usize]
            } else {
                vec!['<'; (pos.x - dest.x) as usize]
            };

            let y_moves = if pos.y > dest.y {
                vec!['^'; (pos.y - dest.y) as usize]
            } else {
                vec!['v'; (dest.y - pos.y) as usize]
            };

            // figure out how to get there. We do not want to hit the gap
            if self.gap.y == pos.y {
                moves.extend(y_moves);
                moves.extend(x_moves);
            } else {
                moves.extend(x_moves);
                moves.extend(y_moves);
            }

            moves.push('A');

            pos = dest;
        }

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

    let keypad = KeyPad::new_button_pad();
    let arrow_pad = KeyPad::new_arrow_pad();

    Ok(input
        .inputs
        .iter()
        .map(|code| {
            let number = code_number(code);

            let mut c = keypad.short_key_path(code);
            tracing::info!("DEBUG: {}", code);
            for _ in 0..3 {
                tracing::info!("    NEXT: {}", c);
                c = arrow_pad.short_key_path(&c);
            }

            let mincode =
                arrow_pad.short_key_path(&arrow_pad.short_key_path(&keypad.short_key_path(code)));

            tracing::info!("{}: CODE {} and mincode {}", code, number, mincode.len());

            number * mincode.len()
        })
        .sum())
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    Ok(0)
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

    #[test]
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
