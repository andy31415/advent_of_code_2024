use glam::IVec2;
use nom::{
    character::complete::line_ending,
    multi::{many1, separated_list1},
    IResult, Parser,
};
use std::{collections::HashMap, fmt::Debug};

/// Represents a rectangular map of values that is maintained
/// as a hash-map
#[derive(Debug, Clone, PartialEq)]
pub struct Map<T: Clone + PartialEq + Debug> {
    rows: usize,
    cols: usize,
    values: HashMap<IVec2, T>,
}

impl<T: Clone + PartialEq + Debug> Default for Map<T> {
    fn default() -> Self {
        Self {
            rows: 0,
            cols: 0,
            values: Default::default(),
        }
    }
}

pub trait Parseable {
    type Item;
    fn parse(s: &str) -> IResult<&str, Self::Item>;
}

impl<T: Clone + PartialEq + Debug> Map<T> {
    pub fn is_inside(&self, pos: IVec2) -> bool {
        pos.x >= 0 && (pos.x as usize) < self.cols && pos.y >= 0 && (pos.y as usize) < self.rows
    }

    pub fn row_count(&self) -> usize {
        self.rows
    }

    pub fn col_count(&self) -> usize {
        self.cols
    }

    pub fn get(&self, pos: &IVec2) -> Option<&T> {
        self.values.get(pos)
    }

    pub fn get_mut(&mut self, pos: &IVec2) -> Option<&mut T> {
        self.values.get_mut(pos)
    }
}

impl<T: PartialEq + Debug + Copy + Clone + Parseable<Item = T>> Map<T> {
    /// Parses the map from a newline-separated list of items
    /// The underlying type generally is parseable from char, but could be multi-char as well (e.g.
    /// integers separated by space)
    pub fn parse(input: &str) -> IResult<&str, Self> {
        separated_list1(
            line_ending,
            many1(T::parse)
                .map(|row_values| row_values.into_iter().enumerate().collect::<Vec<_>>()),
        )
        .map(|all| {
            all.iter().fold(Map::default(), |mut m, v| {
                match m.cols {
                    0 => m.cols = v.len(),
                    _ => assert_eq!(
                        m.cols,
                        v.len(),
                        "All rows of a map must be of the same length"
                    ),
                }
                for (x, value) in v {
                    m.values
                        .insert(IVec2::new(*x as i32, m.rows as i32), *value);
                }
                m.rows += 1;
                m
            })
        })
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use nom::{
        character::complete::{self, space0},
        combinator::fail,
    };
    use nom_supreme::ParserExt;

    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum SomeChar {
        Value(char),
    }

    impl Parseable for SomeChar {
        type Item = SomeChar;

        #[tracing::instrument]
        fn parse(s: &str) -> IResult<&str, Self::Item> {
            match s.char_indices().next() {
                // ignore newlines
                Some((_, '\n')) | Some((_, '\r')) => fail(s),
                // other chars are fair game
                Some((_, ch)) => {
                    let (_, rest) = s.split_at(ch.len_utf8());
                    Ok((rest, SomeChar::Value(ch)))
                }
                None => fail(s),
            }
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn parsing_chars() {
        assert_eq!(
            Map::<SomeChar>::parse("123\nabc").expect("valid input").1,
            Map {
                rows: 2,
                cols: 3,
                values: HashMap::from_iter(vec![
                    (IVec2::new(0, 0), SomeChar::Value('1')),
                    (IVec2::new(1, 0), SomeChar::Value('2')),
                    (IVec2::new(2, 0), SomeChar::Value('3')),
                    (IVec2::new(0, 1), SomeChar::Value('a')),
                    (IVec2::new(1, 1), SomeChar::Value('b')),
                    (IVec2::new(2, 1), SomeChar::Value('c')),
                ]),
            }
        );
    }

    impl Parseable for u32 {
        type Item = u32;

        #[tracing::instrument]
        fn parse(s: &str) -> IResult<&str, Self::Item> {
            complete::u32.terminated(space0).parse(s)
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn parsing_numbers() {
        assert_eq!(
            Map::<u32>::parse("1 2\n10 20\n123 321")
                .expect("valid input")
                .1,
            Map {
                rows: 3,
                cols: 2,
                values: HashMap::from_iter(vec![
                    (IVec2::new(0, 0), 1),
                    (IVec2::new(1, 0), 2),
                    (IVec2::new(0, 1), 10),
                    (IVec2::new(1, 1), 20),
                    (IVec2::new(0, 2), 123),
                    (IVec2::new(1, 2), 321),
                ]),
            }
        );
    }
}
