use std::{fmt::Debug, ops::Add};

#[derive(Debug, PartialEq, Eq, Clone)]
struct Matrix {
    rows: Vec<Vec<char>>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct CharsIterator<'a> {
    matrix: &'a Matrix,
    current: Point,
    direction: &'a Heading,
}

impl Iterator for CharsIterator<'_> {
    type Item = char;

    //#[tracing::instrument(ret)]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.matrix.at(&self.current);
        if value.is_some() {
            self.current = self.current + self.direction.direction();
        }
        value
    }
}

struct PointsIterator<'a> {
    matrix: &'a Matrix,
    current: Point,
}

impl<'a> PointsIterator<'a> {
    fn new(matrix: &'a Matrix) -> PointsIterator<'a> {
        PointsIterator {
            matrix,
            current: Point { x: 0, y: 0 },
        }
    }
}

impl Iterator for PointsIterator<'_> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        // find out if current is real
        self.matrix.at(&self.current)?;

        let retval = self.current;

        // try to advance current
        let row = &self.matrix.rows[self.current.y as usize];
        if ((self.current.x + 1) as usize) < row.len() {
            self.current.x += 1;
        } else {
            self.current.y += 1;
            self.current.x = 0;
        }

        Some(retval)
    }
}

impl<'a> Matrix {
    fn at(&self, p: &Point) -> Option<char> {
        if p.y < 0 || p.y as usize >= self.rows.len() {
            return None;
        }
        let row = &self.rows[p.y as usize];

        if p.x < 0 || p.x as usize >= row.len() {
            return None;
        }
        Some(row[p.x as usize])
    }

    fn points(&self) -> PointsIterator {
        PointsIterator::new(self)
    }

    fn chars_at(&'a self, p: Point, d: &'a Heading) -> CharsIterator<'a> {
        CharsIterator {
            matrix: self,
            current: p,
            direction: d,
        }
    }

    fn is_xmas_at(&'a self, p: &Point) -> bool {
        match self.at(p) {
            Some('A') => {}
            _ => return false, // A is in the middle
        }

        // is XMAS if M & S exist
        for d in [Heading::N, Heading::NE] {
            // one direction has to be MAS
            match self
                .chars_at(*p + d.direction(), &d.flip())
                .take(3)
                .collect::<String>()
                .as_str()
            {
                "MAS" | "SAM" => {}
                _ => continue,
            }

            let other = d.turn_right_90();

            match self
                .chars_at(*p + other.direction(), &other.flip())
                .take(3)
                .collect::<String>()
                .as_str()
            {
                "MAS" | "SAM" => {}
                _ => continue,
            }
            tracing::info!("FOUND AT {:?} via {:?}", p, d);

            return true;
        }
        false
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Heading {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Heading {
    fn all() -> Vec<Heading> {
        vec![
            Heading::N,
            Heading::NE,
            Heading::E,
            Heading::SE,
            Heading::S,
            Heading::SW,
            Heading::W,
            Heading::NW,
        ]
    }

    // flip at 90 degrees
    fn turn_right_90(&self) -> Heading {
        match self {
            Heading::N => Heading::E,
            Heading::NE => Heading::SE,
            Heading::E => Heading::S,
            Heading::SE => Heading::SW,
            Heading::S => Heading::W,
            Heading::SW => Heading::NW,
            Heading::W => Heading::N,
            Heading::NW => Heading::NE,
        }
    }

    // flip the direction
    fn flip(&self) -> Heading {
        match self {
            Heading::N => Heading::S,
            Heading::NE => Heading::SW,
            Heading::E => Heading::W,
            Heading::SE => Heading::NW,
            Heading::S => Heading::N,
            Heading::SW => Heading::NE,
            Heading::W => Heading::E,
            Heading::NW => Heading::SE,
        }
    }

    fn direction(&self) -> Point {
        match self {
            Heading::N => Point { x: 0, y: -1 },
            Heading::NE => Point { x: 1, y: -1 },
            Heading::E => Point { x: 1, y: 0 },
            Heading::SE => Point { x: 1, y: 1 },
            Heading::S => Point { x: 0, y: 1 },
            Heading::SW => Point { x: -1, y: 1 },
            Heading::W => Point { x: -1, y: 0 },
            Heading::NW => Point { x: -1, y: -1 },
        }
    }
}

mod parse {
    use super::Matrix;
    use nom::{
        character::{self, complete::newline},
        multi::{many0, many1, separated_list0},
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    pub(crate) fn input_matrix(s: &str) -> IResult<&str, Matrix> {
        separated_list0(newline, many1(character::complete::none_of("\r\n")))
            .terminated(many0(newline))
            .map(|rows| Matrix { rows })
            .parse(s)
    }
}

pub fn part1(input: &str) -> usize {
    let (r, m) = parse::input_matrix(input).expect("Parsing is ok");
    assert_eq!(r, "");

    // find all that match XMAS
    let mut count = 0;
    for start_pos in m.points() {
        for d in Heading::all() {
            let items = m.chars_at(start_pos, &d).take(4).collect::<String>();

            if items == "XMAS" {
                count += 1;
            }
        }
    }
    count
}

pub fn part2(input: &str) -> usize {
    let (r, m) = parse::input_matrix(input).expect("Parsing is ok");
    assert_eq!(r, "");

    // find all that match XMAS
    let mut count = 0;
    for start_pos in m.points() {
        if m.is_xmas_at(&start_pos) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_iter() {
        let (r, m) = super::parse::input_matrix("ABC\n123\nXYZ").expect("parse works");
        assert_eq!(r, "");

        assert_eq!(
            m.chars_at(Point { x: 0, y: 0 }, &Heading::S).collect_vec(),
            vec!['A', '1', 'X'],
        );

        assert_eq!(
            m.chars_at(Point { x: 0, y: 0 }, &Heading::E).collect_vec(),
            vec!['A', 'B', 'C'],
        );

        assert_eq!(
            m.chars_at(Point { x: 0, y: 0 }, &Heading::N).collect_vec(),
            vec!['A'],
        );

        assert_eq!(
            m.chars_at(Point { x: 1, y: 1 }, &Heading::NW).collect_vec(),
            vec!['2', 'A'],
        );
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 18);
    }

    #[test]
    fn test_xmas() {
        let (r, m) = super::parse::input_matrix("M.S\n.A.\nM.S").expect("parse works");
        assert!(r.is_empty());
        assert!(m.is_xmas_at(&Point { x: 1, y: 1 }));

        let (r, m) = super::parse::input_matrix(".M.\nMAS\n.S.").expect("parse works");
        assert!(r.is_empty());
        assert!(m.is_xmas_at(&Point { x: 1, y: 1 }));
        assert!(!m.is_xmas_at(&Point { x: 0, y: 0 }));

        let (r, m) = super::parse::input_matrix(".S.\nMAS\n.M.").expect("parse works");
        assert!(r.is_empty());
        assert!(m.is_xmas_at(&Point { x: 1, y: 1 }));
        assert!(!m.is_xmas_at(&Point { x: 0, y: 0 }));
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 9);
    }
}
