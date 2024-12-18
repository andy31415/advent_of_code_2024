use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    multi::{many0, many1, separated_list1},
    sequence::{self, tuple},
    Parser as _,
};
use nom_supreme::ParserExt;

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
    fn value(&self) -> u64 {
        let LiteralOperand::Value(v) = self;
        *v as u64
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
    values: [u64; 3],
    pc: usize,
}

impl Registers {
    fn new(values: [u64; 3], pc: usize) -> Self {
        Self { values, pc }
    }

    fn RegisterA(&self) -> u64 {
        self.values[0]
    }
    fn RegisterB(&self) -> u64 {
        self.values[1]
    }
    fn RegisterC(&self) -> u64 {
        self.values[2]
    }
    fn ProgramCounter(&self) -> usize {
        self.pc
    }
}

impl ComboOperand {
    fn value(&self, registers: &[u64]) -> u64 {
        match self {
            ComboOperand::Value(x) => *x as u64,
            ComboOperand::Register(n) => registers[*n as usize],
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
    fn perform(&mut self, instruction: Instruction) -> Option<u64> {
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

#[derive(Default, Debug)]
struct Program {
    registers: Registers,
    instructions: Vec<Instruction>,
}

impl Program {
    fn run(&mut self) -> Vec<u64> {
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
        complete::u64
            .preceded_by(tag("Register A: "))
            .terminated(line_ending),
        complete::u64
            .preceded_by(tag("Register B: "))
            .terminated(line_ending),
        complete::u64
            .preceded_by(tag("Register C: "))
            .terminated(many1(line_ending)),
        separated_list1(tag(","), complete::u8)
            .preceded_by(tag("Program: "))
            .terminated(many0(line_ending)),
    ))
    .map(|(a, b, c, instruction_vec)| Program {
        registers: Registers::new([a, b, c], 0),
        instructions: instruction_vec
            .chunks_exact(2)
            .map(|v| Instruction::try_from(v).expect("valid instruction"))
            .collect(),
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

pub fn part1(input: &str) -> color_eyre::Result<Vec<u64>> {
    let mut program = parse_input(input)?;

    Ok(program.run())
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
    fn test_operations() {
        {
            let mut r = Registers::new([0, 0, 9], 0);
            assert_eq!(
                r.perform([2u8, 6u8].as_ref().try_into().expect("valid instruction")),
                None
            );
            assert_eq!(r.RegisterB(), 1);
            assert_eq!(r.ProgramCounter(), 1);
        }

        {
            let mut program = Program {
                registers: Registers::new([10, 0, 0], 0),
                instructions: [5, 0, 5, 1, 5, 4]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
            };

            assert_eq!(program.run(), vec![0, 1, 2]);
            assert_eq!(program.registers.ProgramCounter(), 3);
        }

        {
            let mut program = Program {
                registers: Registers::new([2024, 0, 0], 0),
                instructions: [0, 1, 5, 4, 3, 0]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
            };

            assert_eq!(program.run(), vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0]);
            assert_eq!(program.registers.RegisterA(), 0);
        }

        {
            let mut program = Program {
                registers: Registers::new([0, 29, 0], 0),
                instructions: [1, 7]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
            };

            assert_eq!(program.run(), vec![]);
            assert_eq!(program.registers.RegisterB(), 26);
        }

        {
            let mut program = Program {
                registers: Registers::new([0, 2024, 43690], 0),
                instructions: [4, 0]
                    .chunks_exact(2)
                    .map(|v| Instruction::try_from(v).expect("valid instruction"))
                    .collect(),
            };

            assert_eq!(program.run(), vec![]);
            assert_eq!(program.registers.RegisterB(), 44354);
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
        assert_eq!(part2(include_str!("../example.txt")).expect("success"), 0);
    }
}
