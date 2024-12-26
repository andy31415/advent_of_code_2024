use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::atomic::{AtomicU32, Ordering},
};

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
use rayon::prelude::*;

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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
    max_depth: usize,
) -> Option<bool> {
    if max_depth == 0 {
        return None;
    }
    if let Some(value) = inputs.get(v) {
        return Some(*value);
    }

    // need to find the underlying value instead
    match gate_map.get(v) {
        Some(OperationMapping {
            op1,
            op2,
            operation,
        }) => {
            let v1 = solve(op1, inputs, gate_map, max_depth - 1)?;
            let v2 = solve(op2, inputs, gate_map, max_depth - 1)?;
            Some(match operation {
                Operation::And => v1 && v2,
                Operation::Or => v1 || v2,
                Operation::Xor => v1 ^ v2,
            })
        }
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
        match solve(&z, &mut input.inputs, &input.gate_map, 89) {
            Some(true) => result += 1,
            Some(false) => {}
            None => panic!("invalid linkages"),
        }
    }

    Ok(result)
}

#[derive(Clone, PartialEq)]
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

    fn swap_outputs(&mut self, g1: &str, g2: &str) {
        let m1 = self.gate_map.get(g1).unwrap().clone();
        let m2 = self.gate_map.get(g2).unwrap().clone();

        self.gate_map.insert(g1.to_string(), m2);
        self.gate_map.insert(g2.to_string(), m1);
    }

    fn outpus_involved(&self, gate: &String) -> HashSet<String> {
        let mut result = HashSet::new();

        fn gather_outputs(e: &Executer, s: &String, out: &mut HashSet<String>) {
            if out.contains(s) {
                return;
            }

            if let Some(operation) = e.gate_map.get(s) {
                out.insert(s.clone());
                gather_outputs(e, &operation.op1, out);
                gather_outputs(e, &operation.op2, out);
            }
        }
        gather_outputs(self, gate, &mut result);

        result
    }

    fn exec(&self, x: usize, y: usize) -> Option<usize> {
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
        for id in (0..self.z_bits).into_iter().rev() {
            result <<= 1;
            let key = format!("z{:02}", id);
            match solve(&key, &mut inputs, &self.gate_map, 89) {
                Some(true) => result += 1,
                Some(false) => {}
                None => return None,
            }
        }

        Some(result)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct SwapDest {
    a: String,
    b: String,
}

pub fn part2(input: &str) -> color_eyre::Result<String> {
    let input = parse_input(input)?;

    let x_bits = input.inputs.keys().filter(|k| k.starts_with("x")).count();
    let y_bits = input.inputs.keys().filter(|k| k.starts_with("y")).count();
    let z_bits = input.gate_map.keys().filter(|k| k.starts_with("z")).count();

    let mut executer = Executer::from(x_bits, y_bits, z_bits, &input.gate_map);

    // find out where the first error occurs
    //
    let mut bad_outputs = HashSet::new();
    let mut good_outputs = HashSet::new();

    for bit in 1..x_bits {
        let a = 1 << bit;
        let b = 1 << (bit - 1);

        // ok IFF both carry and sum are ok
        let s1 = executer.exec(a, 0).unwrap();
        let s2 = executer.exec(b, b).unwrap();
        if (s1 == a) && (s2 == a) {
            println!("BIT {:2} IS OK", bit);
            good_outputs.extend(executer.outpus_involved(&format!("z{:02}", bit)));
        } else {
            //test_bits.push(bit);
            println!("BIT {:2} SEEMS BAD", bit);
            bad_outputs.extend(executer.outpus_involved(&format!("z{:02}", bit)));
        }
    }

    let all_outputs = executer.gate_map.keys().cloned().collect::<HashSet<_>>();

    let bad_outputs = bad_outputs
        .iter()
        .filter(|x| !x.starts_with('x') && !x.starts_with('y'))
        .sorted()
        .collect::<Vec<_>>();

    // SOLUTION values
    //   bmn,jss,mvb,rds,wss,z08,z18,z23

    // try to swap them and see what happens

    println!("BAD OUTPUTS: {}: {:?}", bad_outputs.len(), bad_outputs);

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
}
