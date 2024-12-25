use std::collections::HashMap;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{line_ending, space1},
    multi::{many0, many1, separated_list1},
    sequence::tuple,
    IResult, Parser,
};
use nom_supreme::ParserExt;

#[derive(thiserror::Error, Debug, PartialEq)]
enum ProcessingError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
enum Operation {
    And,
    Or,
    Xor,
}

#[derive(Debug)]
struct Gate {
    op1: String,
    op2: String,
    operation: Operation,
    output: String,
}

#[derive(Debug, Clone)]
struct OperationMapping {
    op1: String,
    op2: String,
    operation: Operation,
}

#[derive(Debug)]
struct Input {
    inputs: HashMap<String, bool>, // 0 == false, 1 == true
    gate_map: HashMap<String, OperationMapping>,
}

impl Gate {
    fn mapping(&self) -> OperationMapping {
        OperationMapping {
            op1: self.op1.clone(),
            op2: self.op2.clone(),
            operation: self.operation,
        }
    }
}

fn parse_operand(s: &str) -> IResult<&str, String> {
    is_not("\n\r: ").map(|s: &str| s.to_string()).parse(s)
}

fn parse_input_value(s: &str) -> IResult<&str, (String, bool)> {
    tuple((
        parse_operand,
        tag(": "),
        alt((tag("0").value(false), tag("1").value(true))),
    ))
    .map(|(op, _, value)| (op, value))
    .parse(s)
}

fn parse_gate(s: &str) -> IResult<&str, Gate> {
    tuple((
        parse_operand.terminated(space1),
        alt((
            tag("AND").value(Operation::And),
            tag("OR").value(Operation::Or),
            tag("XOR").value(Operation::Xor),
        ))
        .terminated(space1),
        parse_operand.terminated(space1),
        parse_operand.preceded_by(tag("-> ")),
    ))
    .map(|(op1, operation, op2, output)| Gate {
        op1,
        op2,
        operation,
        output,
    })
    .parse(s)
}

fn parse_input(s: &str) -> Result<Input, ProcessingError> {
    let (rest, input) = tuple((
        separated_list1(line_ending, parse_input_value).terminated(many1(line_ending)),
        separated_list1(line_ending, parse_gate).terminated(many0(line_ending)),
    ))
    .map(|(input_vec, gates)| {
        let mut inputs = HashMap::new();
        for (k, v) in input_vec {
            inputs.insert(k, v);
        }

        let mut gate_map = HashMap::new();
        for g in gates.iter() {
            gate_map.insert(g.output.clone(), g.mapping());
        }

        Input { inputs, gate_map }
    })
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

#[tracing::instrument(ret, skip(inputs, gate_map))]
fn solve(
    v: &str,
    inputs: &mut HashMap<String, bool>,
    gate_map: &HashMap<String, OperationMapping>,
) -> bool {
    if let Some(value) = inputs.get(v) {
        return *value;
    }

    // need to find the underlying value instead
    match gate_map.get(v) {
        Some(OperationMapping {
            op1,
            op2,
            operation,
        }) => match operation {
            Operation::And => solve(op1, inputs, gate_map) && solve(op2, inputs, gate_map),
            Operation::Or => solve(op1, inputs, gate_map) || solve(op2, inputs, gate_map),
            Operation::Xor => solve(op1, inputs, gate_map) ^ solve(op2, inputs, gate_map),
        },
        None => panic!("Output {} should have has a gate connected to it", v),
    }
}

pub fn part1(input: &str) -> color_eyre::Result<usize> {
    let mut input = parse_input(input)?;

    let z_outs = input
        .gate_map
        .keys()
        .filter(|k| k.starts_with("z"))
        .sorted()
        .rev()
        .cloned()
        .collect::<Vec<_>>();

    let mut result = 0;
    for z in z_outs {
        result <<= 1;
        if solve(&z, &mut input.inputs, &input.gate_map) {
            result += 1;
        }
    }

    Ok(result)
}

struct Executer {
    x_bits: usize,
    y_bits: usize,
    z_bits: usize,
    gate_map: HashMap<String, OperationMapping>,
}

impl Executer {
    fn from(
        x_bits: usize,
        y_bits: usize,
        z_bits: usize,
        gate_map: &HashMap<String, OperationMapping>,
    ) -> Self {
        Self {
            x_bits,
            y_bits,
            z_bits,
            gate_map: gate_map.clone(),
        }
    }

    fn exec(&self, x: usize, y: usize) -> usize {
        let mut inputs = HashMap::new();

        for id in 0..self.x_bits {
            let key = format!("x{:02}", id);
            inputs.insert(key, ((x >> id) & 0x01) != 0);
        }
        for id in 0..self.y_bits {
            let key = format!("y{:02}", id);
            inputs.insert(key, ((y >> id) & 0x01) != 0);
        }

        let mut result = 0;
        for id in 0..self.z_bits {
            result <<= 1;
            let key = format!("z{:02}", id);
            if solve(&key, &mut inputs, &self.gate_map) {
                result += 1;
            }
        }

        result
    }
}

pub fn part2(input: &str) -> color_eyre::Result<String> {
    let input = parse_input(input)?;

    let x_bits = input.inputs.keys().filter(|k| k.starts_with("x")).count();
    let y_bits = input.inputs.keys().filter(|k| k.starts_with("y")).count();
    let z_bits = input.gate_map.keys().filter(|k| k.starts_with("z")).count();

    let adder = Executer::from(x_bits, y_bits, z_bits, &input.gate_map);

    for shift in 0..x_bits {
        let a = 1 << shift;
        let b = adder.exec(a, 0);

        if a == b {
            println!("OK AT {}", shift);
        }

        let b = adder.exec(0, a);
        if a == b {
            println!("OK AT {}", shift);
        }
    }

    Ok("".to_string())
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

    #[test_log::test]
    fn test_part1() {
        init_tests();
        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            2024
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(
            part2(include_str!("../example.txt")).expect("success"),
            "z00,z01,z02,z05"
        );
    }
}
