use std::collections::{HashMap, HashSet};

enum Heading {
    N,
    E,
    S,
    W,
}

#[derive(Debug, PartialEq, Eq, Default)]
struct Lab {
    walls: HashSet<(u32, u32)>, // (row,column) where a `#` exists
    rows: u32,
    cols: u32,
    start: (u32, u32),
}

mod parse {
    use nom::{
        character::complete::{line_ending, one_of},
        combinator::opt,
        multi::{fold_many0, fold_many1, many0},
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
    #[tracing::instrument]
    pub(crate) fn row(s: &str) -> IResult<&str, ParsedRow> {
        fold_many1(
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
        fold_many0(row, Lab::default, |mut lab: Lab, row| {
            let y = lab.rows;

            if lab.cols == 0 {
                lab.cols = row.length;
            } else {
                assert_eq!(lab.cols, row.length);
            }

            if let Some(start_x) = row.start {
                assert_eq!(lab.start, (0, 0));
                lab.start = (y, start_x);
            }

            // add all walls
            for x in row.walls {
                lab.walls.insert((y, x));
            }
            lab.rows += 1;
            lab
        })
        .parse(s)
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
    fn test_input() {
        assert_eq!(
            input("#.#\n..#\n^..\n...").expect("valid input").1,
            Lab {
                cols: 3,
                rows: 4,
                walls: [(0, 0), (0, 2), (1, 2)].into(),
                start: (2, 0)
            }
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
