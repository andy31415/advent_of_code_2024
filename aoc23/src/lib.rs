use std::collections::{HashMap, HashSet};

use nom::{
    bytes::complete::{is_not, tag},
    character::complete::line_ending,
    multi::{many0, separated_list1},
    sequence::separated_pair,
    Parser,
};
use nom_supreme::ParserExt;
use petgraph::graph::{NodeIndex, UnGraph};

#[derive(thiserror::Error, Debug, PartialEq)]
enum ProcessingError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

struct Input {
    node_indexes: HashMap<String, NodeIndex<u32>>,
    graph: UnGraph<String, ()>,
}

fn parse_input(s: &str) -> Result<Input, ProcessingError> {
    let mut node_indexes = HashMap::new();
    let mut graph = UnGraph::<String, ()>::default();

    let (rest, items) = separated_list1(
        line_ending,
        separated_pair(is_not("\n\r-"), tag("-"), is_not("\n\r-")),
    )
    .terminated(many0(line_ending))
    .parse(s)?;

    for (a, b) in items {
        let a_idx = *node_indexes
            .entry(a.to_string())
            .or_insert_with(|| graph.add_node(a.to_string()));
        let b_idx = *node_indexes
            .entry(b.to_string())
            .or_insert_with(|| graph.add_node(b.to_string()));

        graph.add_edge(a_idx, b_idx, ());
    }

    if !rest.is_empty() {
        return Err(ProcessingError::UnparsedData(rest.into()));
    }

    Ok(Input {
        node_indexes,
        graph,
    })
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for ProcessingError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        ProcessingError::NomError(value.map_input(|i| i.into()))
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    let mut interconnected = HashSet::new();

    for (a_name, a_idx) in input.node_indexes.iter() {
        for b_idx in input.graph.neighbors(*a_idx) {
            let b_name = input.graph.node_weight(b_idx).expect("valid");
            for c_idx in input.graph.neighbors(b_idx).filter(|idx| idx != a_idx) {
                let c_name = input.graph.node_weight(c_idx).expect("valid");

                // C must be connected to a as well
                if input.graph.find_edge(*a_idx, c_idx).is_none() {
                    continue;
                }

                if !a_name.starts_with("t") && !b_name.starts_with("t") && !c_name.starts_with("t")
                {
                    continue;
                }

                let mut v = vec![a_name.clone(), b_name.clone(), c_name.clone()];
                v.sort();

                interconnected.insert(v);
            }
        }
    }

    Ok(interconnected.len())
}

pub fn part2(input: &str) -> color_eyre::Result<usize> {
    let input = parse_input(input)?;

    todo!();
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
        assert_eq!(part1(include_str!("../example.txt")).expect("success"), 7);
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
