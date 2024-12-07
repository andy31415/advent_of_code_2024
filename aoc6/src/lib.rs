use std::{collections::HashSet, ops::Add};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Copy, Clone)]
enum Heading {
    N,
    E,
    S,
    W,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Copy, Clone)]
struct Point {
    row: i32,
    col: i32,
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl From<Point> for (i32, i32) {
    fn from(value: Point) -> Self {
        (value.row, value.col)
    }
}

impl From<(i32, i32)> for Point {
    fn from(value: (i32, i32)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

impl Add<(i32, i32)> for Point {
    type Output = Point;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Point {
            row: self.row + rhs.0,
            col: self.col + rhs.1,
        }
    }
}

impl Heading {
    fn direction(&self) -> (i32, i32) {
        match self {
            Heading::N => (-1, 0),
            Heading::E => (0, 1),
            Heading::S => (1, 0),
            Heading::W => (0, -1),
        }
    }

    fn turn_right(&self) -> Self {
        match self {
            Heading::N => Heading::E,
            Heading::E => Heading::S,
            Heading::S => Heading::W,
            Heading::W => Heading::N,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
struct Lab {
    walls: HashSet<(i32, i32)>, // (row,column) where a `#` exists
    rows: i32,
    cols: i32,
    start: (i32, i32),
}

impl Lab {
    fn contains(&self, pos: (i32, i32)) -> bool {
        (pos.0 >= 0) && (pos.0 < self.rows) && (pos.1 >= 0) && (pos.1 < self.cols)
    }
}

mod parse {
    use nom::{
        character::complete::{line_ending, one_of},
        combinator::opt,
        multi::{fold_many0, fold_many1},
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    use crate::Lab;

    #[derive(Debug, PartialEq, Eq, Default)]
    pub(crate) struct ParsedRow {
        length: i32,        // full row length
        walls: Vec<i32>,    // where walls are located
        start: Option<i32>, // where ^ is located
    }

    impl ParsedRow {
        pub(crate) fn new<W: Into<Vec<i32>>>(length: i32, walls: W, start: Option<i32>) -> Self {
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

fn display_path(lab: &Lab, visited: &HashSet<Point>) -> String {
    let mut s = String::new();
    s.reserve((lab.rows * (lab.cols + 1)) as usize);

    for r in 0..lab.rows {
        for c in 0..lab.cols {
            if lab.walls.contains(&(r, c)) {
                s.push('#');
            } else if visited.contains(&(r, c).into()) {
                s.push('X');
            } else {
                s.push('.');
            }
        }
        s.push('\n');
    }

    s
}

// hahsset: what was visited, bool: stuck in a loop or not
pub fn find_size(lab: &Lab) -> (HashSet<Point>, bool) {
    // '^' means heading north
    let mut position: (Point, Heading) = (lab.start.into(), Heading::N);

    let mut visited = HashSet::new();
    let mut positions = HashSet::new();

    tracing::info!("LAB {:#?}", lab);

    while !positions.contains(&position) {
        tracing::info!("at {:?}", position);
        positions.insert(position);
        visited.insert(position.0);

        let next = position.0 + position.1.direction();
        tracing::info!("  Trying to move to {:?}", next);
        if !lab.contains(next.into()) {
            tracing::info!("  Exiting lab");
            // tracing::info!("PATH SO FAR:\n{}", display_path(&lab, &visited));
            // moved outside the lab
            return (visited, false);
        }

        // position is in the lab ... are we hitting anything?
        if lab.walls.contains(&next.into()) {
            tracing::info!("  Would hit a wall, so turn right");
            // we have to turn because otherwise we hit a wall
            position.1 = position.1.turn_right();

            tracing::info!("PATH SO FAR:\n{}", display_path(lab, &visited));
        } else {
            tracing::info!("  Can move");
            position.0 = next; // move, keeping the same heading
        }
    }
    // stuck in a loop
    (visited, true)
}

pub fn part1(input: &str) -> usize {
    let (r, lab) = parse::input(input).expect("valid input");
    assert!(r.is_empty());
    let (visited, _) = find_size(&lab);
    visited.len()
}

pub fn part2(input: &str) -> usize {
    let (r, lab) = parse::input(input).expect("valid input");
    assert!(r.is_empty());

    let (initial_visisted, _) = find_size(&lab);

    // try to place an obstacle in all visisted places and see if we go into some loop.
    // Obstacle only makes sense in visisted (otherwise we do not block any path really)
    initial_visisted
        .par_iter()
        .map(|point| {
            if point == &lab.start.into() {
                return 0;
            }

            let mut changed_lab = lab.clone();
            changed_lab.walls.insert((*point).into());

            // check if now we loop
            if let (_, true) = find_size(&changed_lab) {
                1
            } else {
                0
            }
        })
        .sum()
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
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 41);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 6);
    }
}
