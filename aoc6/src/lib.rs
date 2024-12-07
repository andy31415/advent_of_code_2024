use std::collections::{HashMap, HashSet};

struct Lab {
    walls: HashSet<(u32, u32)>, // (row,column) where a `#` exists
    rows: u32,
    cols: u32,
}

mod parse {
    use nom::{
        character::complete::{line_ending, one_of},
        combinator::opt,
        multi::{fold_many0, many0},
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    use crate::Lab;

    #[derive(Debug, PartialEq, Eq, Default)]
    pub(crate) struct ParsedRow {
        length: u32,        // full row length
        walls: Vec<u32>,    // where walls are located
        start: Option<u32>, // where ^ is located
    }

    impl ParsedRow {
        pub(crate) fn new<W: Into<Vec<u32>>>(length: u32, walls: W, start: Option<u32>) -> Self {
            Self {
                length,
                walls: walls.into(),
                start,
            }
        }
    }

    /// Returns the length of the row plus all the positions
    pub(crate) fn row(s: &str) -> IResult<&str, ParsedRow> {
        fold_many0(
            one_of(".#^"),
            ParsedRow::default,
            |mut row: ParsedRow, item| {
                match item {
                    '#' => row.walls.push(row.length),
                    '^' => {
                        assert!(row.start.is_none());
                        row.start = Some(row.length);
                    }
                    _ => {} // . is ok, ignore all
                }
                row.length += 1;
                row
            },
        )
        .terminated(opt(line_ending))
        .parse(s)
    }

    pub(crate) fn input(s: &str) -> IResult<&str, Lab> {
        todo!()
    }
}

pub fn part1(input: &str) -> usize {
    // TODO: implement
    0
}

pub fn part2(input: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::parse::{input, row, ParsedRow};
    use super::*;

    #[test]
    fn test_row() {
        assert_eq!(
            row("..##..#.").expect("valid input").1,
            ParsedRow::new(8, [2, 3, 6], None)
        );
        assert_eq!(
            row("........").expect("valid input").1,
            ParsedRow::new(8, [], None)
        );
        assert_eq!(
            row("#..").expect("valid input").1,
            ParsedRow::new(3, [0], None)
        );
        assert_eq!(
            row(".").expect("valid input").1,
            ParsedRow::new(1, [], None)
        );
        assert_eq!(
            row("#").expect("valid input").1,
            ParsedRow::new(1, [0], None)
        );
        assert_eq!(
            row("##").expect("valid input").1,
            ParsedRow::new(2, [0, 1], None)
        );

        // start point
        assert_eq!(
            row("#^#").expect("valid input").1,
            ParsedRow::new(3, [0, 2], Some(1))
        );
        assert_eq!(
            row(".^..").expect("valid input").1,
            ParsedRow::new(4, [], Some(1))
        );

        assert_eq!(
            row("^.").expect("valid input").1,
            ParsedRow::new(2, [], Some(0))
        );
        assert_eq!(
            row("..^").expect("valid input").1,
            ParsedRow::new(3, [], Some(2))
        );
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 41);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
