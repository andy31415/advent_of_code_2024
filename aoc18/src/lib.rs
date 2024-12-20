use std::{collections::HashSet, fmt::Display};

use glam::IVec2;
use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, separated_list1},
    sequence::separated_pair,
    Parser as _,
};
use nom_supreme::ParserExt;
use pathfinding::prelude::dijkstra;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

struct Input {
    positions: Vec<IVec2>,
}

fn parse_input(s: &str) -> Result<Input, InputParseError> {
    let (rest, input) = separated_list1(
        line_ending,
        separated_pair(complete::i32, tag(","), complete::i32).map(|(x, y)| IVec2::new(x, y)),
    )
    .terminated(many0(line_ending))
    .map(|positions| Input { positions })
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(input)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

struct Grid {
    rows: i32,
    cols: i32,
    blocks: HashSet<IVec2>,
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.rows {
            let mut s = String::with_capacity((self.cols + 1) as usize);
            for x in 0..self.cols {
                if self.blocks.contains(&(x, y).into()) {
                    s.push('#');
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
            f.write_str(&s)?;
        }
        Ok(())
    }
}

pub fn part1(input: &str, grid_size: IVec2, simulation: usize) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    // GRID SIZE: 6x6 OR 70x70
    let g = Grid {
        rows: grid_size.y,
        cols: grid_size.x,
        blocks: input.positions.into_iter().take(simulation).collect(),
    };

    let goal = IVec2::new(g.rows - 1, g.cols - 1);

    tracing::info!("GRID:\n{}", g);
    let node_map = dijkstra(
        &IVec2::new(0, 0),
        |start| {
            [(0, 1), (0, -1), (1, 0), (-1, 0)]
                .into_iter()
                .map(|(x, y)| start + IVec2::new(x, y))
                .filter(|p| {
                    p.x >= 0 && p.x < g.cols && p.y >= 0 && p.y < g.rows && !g.blocks.contains(p)
                })
                .map(|p| (p, 1))
                .collect::<Vec<_>>()
        },
        |x| x == &goal,
    );

    tracing::info!("MAP: {:?}", node_map);

    Ok(node_map.map(|(_, len)| len).unwrap_or(0))
}

impl Input {
    fn has_path(&self, grid_size: IVec2, simulation_size: usize) -> bool {
        // GRID SIZE: 6x6 OR 70x70
        let g = Grid {
            rows: grid_size.y,
            cols: grid_size.x,
            blocks: self
                .positions
                .iter()
                .take(simulation_size)
                .copied()
                .collect(),
        };

        let goal = IVec2::new(g.rows - 1, g.cols - 1);

        dijkstra(
            &IVec2::new(0, 0),
            |start| {
                [(0, 1), (0, -1), (1, 0), (-1, 0)]
                    .into_iter()
                    .map(|(x, y)| start + IVec2::new(x, y))
                    .filter(|p| {
                        p.x >= 0
                            && p.x < g.cols
                            && p.y >= 0
                            && p.y < g.rows
                            && !g.blocks.contains(p)
                    })
                    .map(|p| (p, 1))
                    .collect::<Vec<_>>()
            },
            |x| x == &goal,
        )
        .is_some()
    }
}

pub fn part2(input: &str, grid_size: IVec2, simulation: usize) -> color_eyre::Result<IVec2> {
    let input = parse_input(input)?;

    let mut low = 0; // possible
    let mut high = input.positions.len() + 1; // impossible

    while (high - low) > 1 {
        let mid = (high + low) / 2;

        if input.has_path(grid_size, mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    Ok(*input.positions.get(low).expect("valid index"))
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
            part1(include_str!("../example.txt"), (7, 7).into(), 12).expect("success"),
            22
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(
            part2(include_str!("../example.txt"), (7,7).into(), 25).expect("success"),
            IVec2::new(6, 1)
        );
    }
}
