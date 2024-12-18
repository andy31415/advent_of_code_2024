use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, many1, separated_list1},
    sequence::{tuple},
    Parser as _,
};
use nom_supreme::ParserExt;
use rayon::iter::ParallelIterator;
use rayon::prelude::*;

#[derive(thiserror::Error, Debug, PartialEq)]
enum InputParseError {
    #[error("Failed to parse using Nom")]
    NomError(#[source] nom::Err<nom::error::Error<String>>),

    #[error("Unparsed data remained: {0:?}")]
    UnparsedData(String),

    #[error("Operand opcode {0:?} is not valid")]
    InvalidOperandOpcode(u8),

    #[error("Command {0:?} is not valid")]
    InvalidCommand(u8),

    #[error("Failed to decode instructions: exactly 2 bytes needed")]
    InvalidDecodeLength,

    #[error("Part 2 takes a very long time ({0:?} iterations already)")]
    TakesTooLong(u128),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ComboOperand {
    Value(u8),    // 0 to 3 really
    Register(u8), // 0 to 3: 0/A, 1/B, 2/C
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum LiteralOperand {
    Value(u8),
}

impl LiteralOperand {
    fn value(&self) -> u128 {
        let LiteralOperand::Value(v) = self;
        *v as u128
    }
}

impl Display for LiteralOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LiteralOperand::Value(v) = self;
        f.write_fmt(format_args!("{}", v))
    }
}

impl TryFrom<u8> for ComboOperand {
    type Error = InputParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..=3 => Ok(ComboOperand::Value(value)),
            4..=6 => Ok(ComboOperand::Register(value - 4)),
            _ => Err(InputParseError::InvalidOperandOpcode(value)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Registers {
    values: [u128; 3],
    pc: usize,
}

impl Registers {
    fn new(values: [u128; 3], pc: usize) -> Self {
        Self { values, pc }
    }

    fn register_a(&self) -> u128 {
        self.values[0]
    }
    fn register_b(&self) -> u128 {
        self.values[1]
    }
    fn program_counter(&self) -> usize {
        self.pc
    }
}

impl ComboOperand {
    fn value(&self, registers: &[u128]) -> u128 {
        match self {
            ComboOperand::Value(x) => *x as u128,
            ComboOperand::Register(n) => registers[*n as usize],
        }
    }
}

impl Display for ComboOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComboOperand::Value(v) => f.write_fmt(format_args!("{}", v)),
            ComboOperand::Register(r) => match r {
                0 => f.write_str("A"),
                1 => f.write_str("B"),
                2 => f.write_str("C"),
                n => f.write_fmt(format_args!("Register<{}>", n)),
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Instruction {
    DivisionA(ComboOperand), // Register<A/0> = Register<A/0> div 2^OPERAND, truncated integer
    BitwiseXorB(LiteralOperand), // Register<B/1> = Register<B/1> XOR OPERAND
    Modulo8(ComboOperand),   // Register<B/1>   = OPERAND mod 8,
    JumpIfNotZero(LiteralOperand), // Jump to OPERAND IF Register<A/0> is NOT 0
    BitwiseXorC,             // Ignores the operand. Register<B/1> = Register<B/1> XOR Register<C/2>
    Out(ComboOperand),       // OUTPUT OPERAND mod 8
    DivisionB(ComboOperand), // Register<B/1> = Register<A/0> div 2^OPERAND, truncated integer
    DivisionC(ComboOperand), // Register<C/2> = Register<A/0> div 2^OPERAND, truncated integer
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::DivisionA(combo_operand) => {
                f.write_fmt(format_args!("A = A / (1 << {})", combo_operand))
            }
            Instruction::BitwiseXorB(literal_operand) => {
                f.write_fmt(format_args!("B ^= {}", literal_operand))
            }
            Instruction::Modulo8(combo_operand) => {
                f.write_fmt(format_args!("B = {} % 8", combo_operand))
            }
            Instruction::JumpIfNotZero(literal_operand) => {
                f.write_fmt(format_args!("IF A != 0 JMP {}", literal_operand))
            }
            Instruction::BitwiseXorC => f.write_str("B ^= C"),
            Instruction::Out(combo_operand) => {
                f.write_fmt(format_args!("OUT({} % 8)", combo_operand))
            }
            Instruction::DivisionB(combo_operand) => {
                f.write_fmt(format_args!("B = A / (1 << {})", combo_operand))
            }
            Instruction::DivisionC(combo_operand) => {
                f.write_fmt(format_args!("C = A / (1 << {})", combo_operand))
            }
        }
    }
}

impl TryFrom<&[u8]> for Instruction {
    type Error = InputParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(InputParseError::InvalidDecodeLength);
        }
        match value[0] {
            0 => Ok(Self::DivisionA(value[1].try_into()?)),
            1 => Ok(Self::BitwiseXorB(LiteralOperand::Value(value[1]))),
            2 => Ok(Self::Modulo8(value[1].try_into()?)),
            3 => Ok(Self::JumpIfNotZero(LiteralOperand::Value(value[1]))),
            4 => Ok(Self::BitwiseXorC),
            5 => Ok(Self::Out(value[1].try_into()?)),
            6 => Ok(Self::DivisionB(value[1].try_into()?)),
            7 => Ok(Self::DivisionC(value[1].try_into()?)),
            _ => Err(InputParseError::InvalidCommand(value[0])),
        }
    }
}

impl Registers {
    // perform the operation and return an optional outpu
    fn perform(&mut self, instruction: Instruction) -> Option<u128> {
        match instruction {
            Instruction::DivisionA(combo_operand) => {
                self.values[0] /= 1 << combo_operand.value(&self.values);
                self.pc += 1;
            }
            Instruction::BitwiseXorB(literal_operand) => {
                self.values[1] ^= literal_operand.value();
                self.pc += 1;
            }
            Instruction::Modulo8(combo_operand) => {
                self.values[1] = combo_operand.value(&self.values) % 8;
                self.pc += 1;
            }
            Instruction::JumpIfNotZero(literal_operand) => {
                if self.values[0] == 0 {
                    self.pc += 1;
                } else {
                    self.pc = literal_operand.value() as usize;
                }
            }
            Instruction::BitwiseXorC => {
                self.values[1] ^= self.values[2];
                self.pc += 1;
            }
            Instruction::Out(combo_operand) => {
                self.pc += 1;
                return Some(combo_operand.value(&self.values) % 8);
            }
            Instruction::DivisionB(combo_operand) => {
                self.values[1] = self.values[0] / (1 << combo_operand.value(&self.values));
                self.pc += 1;
            }
            Instruction::DivisionC(combo_operand) => {
                self.values[2] = self.values[0] / (1 << combo_operand.value(&self.values));
                self.pc += 1;
            }
        }

        None
    }
}

#[derive(Default, Debug, Clone)]
struct Program {
    registers: Registers,
    instructions: Vec<Instruction>,
    raw_program: Vec<u8>,
}

impl Program {
    fn run(&mut self) -> Vec<u128> {
        let mut output_vec = Vec::new();
        while let Some(instruction) = self.instructions.get(self.registers.pc) {
            if let Some(output) = self.registers.perform(*instruction) {
                output_vec.push(output);
            }
        }
        output_vec
    }
}

fn parse_input(s: &str) -> Result<Program, InputParseError> {
    let (rest, program) = tuple((
        complete::u128
            .preceded_by(tag("Register A: "))
            .terminated(line_ending),
        complete::u128
            .preceded_by(tag("Register B: "))
            .terminated(line_ending),
        complete::u128
            .preceded_by(tag("Register C: "))
            .terminated(many1(line_ending)),
        separated_list1(tag(","), complete::u8)
            .preceded_by(tag("Program: "))
            .terminated(many0(line_ending)),
    ))
    .map(|(a, b, c, raw_program)| Program {
        registers: Registers::new([a, b, c], 0),
        instructions: raw_program
            .chunks_exact(2)
            .map(|v| Instruction::try_from(v).expect("valid instruction"))
            .collect(),
        raw_program,
    })
    .parse(s)?;

    if !rest.is_empty() {
        return Err(InputParseError::UnparsedData(rest.into()));
    }

    Ok(program)
}

impl<INNER: Into<String>> From<nom::Err<nom::error::Error<INNER>>> for InputParseError {
    fn from(value: nom::Err<nom::error::Error<INNER>>) -> Self {
        InputParseError::NomError(value.map_input(|i| i.into()))
    }
}

pub fn part1(input: &str) -> color_eyre::Result<Vec<u128>> {
    let mut program = parse_input(input)?;

    Ok(program.run())
}

#[tracing::instrument(ret)]
pub fn find(start_program: &Program, target: &[u8]) -> Vec<u128> {
    // find the output for the smaller chunks, then append a prefix
    let (first, rest) = match target.split_first() {
        Some((a, b)) => (a, b),
        None => return vec![0], // Empty target
    };

    let mut choices = Vec::new();

    for partial in find(start_program, rest) {
        for suffix in 0..8 {
            let choice = (partial << 3) | suffix;

            let mut other = start_program.clone();
            other.registers.values[0] = choice;
            let result = other.run().iter().map(|v| *v as u8).collect::<Vec<_>>();

            if result == target {
                choices.push(choice);
            }
        }
    }

    choices
}

pub fn part2(input: &str) -> color_eyre::Result<u128> {
    let program = parse_input(input)?;

    tracing::info!("PROGRAM:");
    for (idx, i) in program.instructions.iter().enumerate() {
        tracing::info!("    {}: {:#}", idx, i);
    }

    Ok(*find(
        &program,
        &program
            .raw_program
            .iter().copied()
            .collect::<Vec<_>>(),
    )
    .iter()
    .min()
    .expect("Has a solution"))
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
    fn test_operations() {
        {
            let mut r = Registers::new([0, 0, 9], 0);
            assert_eq!(
                r.perform([2u8, 6u8].as_ref().try_into().expect("valid instruction")),
                None
            );
            assert_eq!(r.register_b(), 1);
            assert_eq!(r.program_counter(), 1);
        }

        {
            let mut program = Program {
                registers: Registers::new([10, 0, 0], 0),
                instructions: [5, 0, 5, 1, 5, 4]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
                raw_program: vec![], // do not care
            };

            assert_eq!(program.run(), vec![0, 1, 2]);
            assert_eq!(program.registers.program_counter(), 3);
        }

        {
            let mut program = Program {
                registers: Registers::new([2024, 0, 0], 0),
                instructions: [0, 1, 5, 4, 3, 0]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
                raw_program: vec![], // do not care
            };

            assert_eq!(program.run(), vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
            assert_eq!(program.registers.register_a(), 0);
        }

        {
            let mut program = Program {
                registers: Registers::new([0, 29, 0], 0),
                instructions: [1, 7]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
                raw_program: vec![], // do not care
            };

            assert_eq!(program.run(), vec![]);
            assert_eq!(program.registers.register_b(), 26);
        }

        {
            let mut program = Program {
                registers: Registers::new([0, 2024, 43690], 0),
                instructions: [4, 0]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
                raw_program: vec![], // do not care
            };

            assert_eq!(program.run(), vec![]);
            assert_eq!(program.registers.register_b(), 44354);
        }
    }

    #[test]
    fn test_part1() {
        init_tests();
        assert_eq!(
            part1(include_str!("../example.txt")).expect("success"),
            vec![4, 6, 3, 5, 6, 3, 5, 2, 1, 0]
        );
    }

    #[test]
    fn test_part2() {
        init_tests();
        assert_eq!(
            part2(include_str!("../example2.txt")).expect("success"),
            117440
        );
    }
}
