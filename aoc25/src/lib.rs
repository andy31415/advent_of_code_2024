use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    multi::{many0, many1, separated_list1},
    IResult, Parser as _,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum ProcessingError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Key {
    Up(Vec<usize>),
    Down(Vec<usize>),
}

#[derive(Debug)]
struct Input {
    keys: Vec<Key>,
}

impl Key {
    fn fit(&self, other: &Key) -> bool {
        let (a, b) = match (self, other) {
            (Key::Up(a), Key::Down(b)) => (a, b),
            (Key::Down(a), Key::Up(b)) => (a, b),
            _ => return false,
        };

        for (x, y) in a.iter().zip(b.iter()) {
            if x + y >= 6 {
                return false;
            }
        }
        true
    }
}

fn parse_key(s: &str) -> IResult<&str, Key> {
    let (rest, lines) = separated_list1(
        line_ending,
        many1(alt((tag("#").value(true), tag(".").value(false)))),
    )
    .parse(s)?;

    let mut v = vec![0, 0, 0, 0, 0];
    assert_eq!(lines.len(), 7);
    assert_eq!(lines.first().unwrap().len(), 5);

    // lines is a vec<vec<bool>>
    if *lines.first().unwrap().first().unwrap() {
        // this is a down item. we assume correct input
        for l in lines.iter().skip(1) {
            for (idx, b) in l.iter().enumerate() {
                if *b {
                    *v.get_mut(idx).unwrap() += 1;
                }
            }
        }
        Ok((rest, Key::Down(v)))
    } else {
        v = vec![6, 6, 6, 6, 6];

        for l in lines.iter() {
            for (idx, b) in l.iter().enumerate() {
                if !*b {
                    *v.get_mut(idx).unwrap() -= 1;
                }
            }
        }

        // this is an up item. we assume correct input
        Ok((rest, Key::Up(v)))
    }
}

fn parse_input(s: &str) -> Result<Input, ProcessingError> {
    let (rest, input) = separated_list1(many1(line_ending), parse_key)
        .terminated(many0(line_ending))
        .map(|keys| Input { keys })
        .parse(s)?;

    if !rest.is_empty() {
        return Err(ProcessingError::UnparsedData(rest.into()));
    }

    Ok(input)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for ProcessingError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        ProcessingError::NomError(value.map_input(|i| i.into()))
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let ups = input
        .keys
        .iter()
        .filter(|k| matches!(k, Key::Up(_)))
        .collect_vec();
    let downs = input
        .keys
        .iter()
        .filter(|k| matches!(k, Key::Down(_)))
        .collect_vec();

    // cartesian product really
    let mut cnt = 0;
    for u in ups.iter() {
        for d in downs.iter() {
            if u.fit(d) {
                println!("FIT: {:?} and {:?}", u, d);
                cnt += 1;
            }
        }
    }

    Ok(cnt)
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
        assert_eq!(part1(include_str!("../example.txt")).expect("success"), 3);
    }
}
