use std::collections::VecDeque;

use itertools::zip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiskLocation {
    File(usize),
    Free(usize),
}

struct FlatDiskIterator {
    current: Option<DiskLocation>,
    rest: VecDeque<DiskLocation>,
    cnt: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct FlatDisk {
    locations: Vec<DiskLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Occupied,
    Free,
}

impl FlatDiskIterator {
    pub fn new(rest: impl Into<VecDeque<DiskLocation>>) -> Self {
        Self {
            current: None,
            cnt: 0,
            rest: rest.into(),
        }
    }

    fn advance(&mut self) {
        // TODO: advance to the next valid one
        if self.cnt > 0 {
            self.cnt -= 1;
        }
        while self.cnt == 0 {
            self.current = self.rest.pop_front();
            self.cnt = self
                .current
                .map(|l| match l {
                    DiskLocation::File(n) => n,
                    DiskLocation::Free(n) => n,
                })
                .unwrap_or(0);

            if self.current.is_none() {
                // we reached the end
                return;
            }
        }
    }
}

impl Iterator for FlatDiskIterator {
    type Item = BlockType;

    fn next(&mut self) -> Option<BlockType> {
        self.advance();

        match self.current {
            None => None,
            Some(value) => match value {
                DiskLocation::File(_) => Some(BlockType::Occupied),
                DiskLocation::Free(_) => Some(BlockType::Free),
            },
        }
    }
}

impl FlatDisk {
    pub fn allocated(&self) -> usize {
        self.locations
            .iter()
            .map(|l| match l {
                DiskLocation::File(n) => *n,
                DiskLocation::Free(_) => 0,
            })
            .sum()
    }

    pub fn blocks(&self) -> FlatDiskIterator {
        FlatDiskIterator::new(self.locations.clone())
    }
}

mod parsing {
    use crate::{DiskLocation, FlatDisk};
    use nom::{
        character::complete::{multispace0, satisfy},
        multi::many0,
        sequence::tuple,
        IResult, Parser,
    };
    use nom_supreme::ParserExt;

    fn occupied_disk(s: &str) -> IResult<&str, DiskLocation> {
        satisfy(|c| c.is_ascii_digit())
            .map(|c| DiskLocation::File(c.to_digit(10).expect("valid digit") as usize))
            .parse(s)
    }

    fn free_disk(s: &str) -> IResult<&str, DiskLocation> {
        satisfy(|c| c.is_ascii_digit())
            .map(|c| DiskLocation::Free(c.to_digit(10).expect("valid digit") as usize))
            .parse(s)
    }

    pub(crate) fn flat_disk(s: &str) -> IResult<&str, FlatDisk> {
        tuple((occupied_disk, many0(tuple((free_disk, occupied_disk)))))
            .map(|(first, rest)| {
                let mut locations = Vec::new();

                locations.push(first);

                for (f, o) in rest {
                    locations.push(f);
                    locations.push(o);
                }

                FlatDisk { locations }
            })
            .terminated(multispace0)
            .parse(s)
    }
}

pub fn part1(input: &str) -> usize {
    let (r, d) = parsing::flat_disk(input).expect("valid input");
    assert!(r.is_empty());

    tracing::info!("Allocated: {}", d.allocated());

    // File IDs only
    let mut blocks = Vec::new();
    let mut id = 0;
    for f in &d.locations {
        if let DiskLocation::File(length) = f {
            for _ in 0..*length {
                blocks.push(id);
            }
            id += 1
        }
    }

    tracing::debug!("BLOCKS: {:?}", blocks);

    let mut fwd = blocks.iter();
    let mut bwd = blocks.iter().rev();

    tracing::debug!("DATA:");

    tracing::debug!(
        "RESULT:  {:?}",
        d.blocks()
            .map(|b| match b {
                BlockType::Occupied => "X",
                BlockType::Free => ".",
            })
            .collect::<Vec<_>>()
    );

    zip(0..d.allocated(), d.blocks())
        .map(|(idx, b)| {
            idx * match b {
                BlockType::Occupied => fwd.next().expect("has value"),
                BlockType::Free => bwd.next().expect("has value"),
            }
        })
        .sum()
}

pub fn part2(input: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parsing::flat_disk("01234").expect("valid").1,
            FlatDisk {
                locations: vec![
                    DiskLocation::File(0),
                    DiskLocation::Free(1),
                    DiskLocation::File(2),
                    DiskLocation::Free(3),
                    DiskLocation::File(4),
                ]
            }
        );
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 1928);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
