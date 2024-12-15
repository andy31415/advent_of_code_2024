use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

use glam::IVec2;
use map_parse::Map;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::opt,
    multi::{many0, many1},
    Parser,
};
use nom_supreme::ParserExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Wall,
    Robot,
    Box,
    Empty,
    LargeBoxLeft,
    LargeBoxRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Left,
    Down,
    Up,
    Right,
}

impl Instruction {
    fn push_direction(&self) -> IVec2 {
        match self {
            Instruction::Left => IVec2::new(-1, 0),
            Instruction::Down => IVec2::new(0, 1),
            Instruction::Up => IVec2::new(0, -1),
            Instruction::Right => IVec2::new(1, 0),
        }
    }
}

impl map_parse::Parseable for Cell {
    type Item = Cell;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        alt((
            tag("#").value(Cell::Wall),
            tag("@").value(Cell::Robot),
            tag("O").value(Cell::Box),
            tag(".").value(Cell::Empty),
        ))
        .parse(s)
    }
}

struct Input {
    map: Map<Cell>,
    instructions: Vec<Instruction>,
    robot_position: IVec2,
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.map.row_count() {
            for x in 0..self.map.col_count() {
                let pos = IVec2::new(x as i32, y as i32);
                if self.robot_position == pos {
                    f.write_char('@')?;
                } else {
                    match self.map.get(&pos).expect("valid location") {
                        Cell::Wall => f.write_char('#'),
                        Cell::Empty => f.write_char('.'),
                        Cell::Box => f.write_char('O'),
                        Cell::Robot => f.write_char('@'),
                        Cell::LargeBoxLeft => f.write_char('['),
                        Cell::LargeBoxRight => f.write_char(']'),
                    }?;
                }
            }
            f.write_char('\n');
        }
        Ok(())
    }
}

impl Input {
    // returns the new robot position if push is ok.

    fn push_object(&mut self, pos: IVec2, direction: IVec2) {
        let value = *self.map.get(&pos).expect("valid position");

        match value {
            Cell::Box => {
                self.push_object(pos + direction, direction);
            }
            Cell::Empty => { /* nothing to move ... */ }
            Cell::LargeBoxLeft => {
                if direction.y != 0 {
                    self.push_object(pos + direction, direction);
                    self.push_object(pos + direction + IVec2::new(1, 0), direction);
                } else {
                    self.push_object(pos + direction, direction)
                }
            }
            Cell::LargeBoxRight => {
                if direction.y != 0 {
                    self.push_object(pos + direction, direction);
                    self.push_object(pos + direction + IVec2::new(-1, 0), direction);
                } else {
                    self.push_object(pos + direction, direction)
                }
            }
            _ => {
                panic!("Unpushable object!");
            }
        }

        let upd = self.map.get_mut(&(pos + direction)).expect("valid");
        assert_eq!(*upd, Cell::Empty);
        *upd = value;
        *self.map.get_mut(&pos).expect("valid") = Cell::Empty;

        // TODO: implement
    }

    fn can_push(&self, pos: IVec2, direction: IVec2) -> bool {
        match self.map.get(&pos).expect("valid position") {
            Cell::Wall => false,
            Cell::Robot => panic!("Robot should not exist in the map directly"),
            Cell::Box => self.can_push(pos + direction, direction),
            Cell::Empty => true,
            Cell::LargeBoxLeft => {
                if direction.y != 0 {
                    self.can_push(pos + direction, direction)
                        && self.can_push(pos + IVec2::new(1, 0) + direction, direction)
                } else {
                    self.can_push(pos + direction, direction)
                }
            }
            Cell::LargeBoxRight => {
                if direction.y != 0 {
                    self.can_push(pos + direction, direction)
                        && self.can_push(pos + IVec2::new(-1, 0) + direction, direction)
                } else {
                    self.can_push(pos + direction, direction)
                }
            }
        }
    }

    fn perform(&mut self, instruction: Instruction) {
        let dir = instruction.push_direction();

        assert_eq!(self.map.get(&self.robot_position), Some(&Cell::Empty));

        if self.can_push(self.robot_position + dir, dir) {
            self.push_object(self.robot_position + dir, dir);
            self.robot_position += dir;
        }
    }

    fn double_horizontally(self) -> Self {
        let rows = self.map.row_count();
        let cols = self.map.col_count() * 2;

        let mut expanded_map = HashMap::new();

        // compute expansion
        for (pos, value) in self.map.values_iter() {
            let p1 = *pos * IVec2::new(2, 1);
            let p2 = p1 + IVec2::new(1, 0);
            match value {
                Cell::Wall => {
                    expanded_map.insert(p1, Cell::Wall);
                    expanded_map.insert(p2, Cell::Wall);
                }
                Cell::Robot => {
                    expanded_map.insert(p1, Cell::Robot);
                    expanded_map.insert(p2, Cell::Empty);
                }
                Cell::Box => {
                    expanded_map.insert(p1, Cell::LargeBoxLeft);
                    expanded_map.insert(p2, Cell::LargeBoxRight);
                }
                Cell::Empty => {
                    expanded_map.insert(p1, Cell::Empty);
                    expanded_map.insert(p2, Cell::Empty);
                }
                _ => panic!("Map cannot be doubled - it looks already doubled."),
            };
        }

        Input {
            map: Map::create(rows, cols, expanded_map),
            instructions: self.instructions,
            robot_position: self.robot_position * IVec2::new(2, 1),
        }
    }
}

fn parse_input(s: &str) -> nom::IResult<&str, Input> {
    let (rest, mut map) = Map::<Cell>::parse(s)?;

    let (rest, instructions) = many1(
        alt((
            tag("^").value(Instruction::Up),
            tag("<").value(Instruction::Left),
            tag(">").value(Instruction::Right),
            tag("v").value(Instruction::Down),
        ))
        .terminated(opt(line_ending)),
    )
    .preceded_by(many0(line_ending))
    .terminated(many0(line_ending))
    .parse(rest)?;

    let mut robot_position = None;
    if let Some((pos, value)) = map
        .values_iter()
        .find(|(pos, value)| **value == Cell::Robot)
    {
        robot_position = Some(*pos);
    }

    if let Some(pos) = robot_position {
        *map.get_mut(&pos).expect("Pos is valid") = Cell::Empty;
    }

    Ok((
        rest,
        Input {
            map,
            instructions,
            robot_position: robot_position.expect("A robot position must exist"),
        },
    ))
}

pub fn part1(s: &str) -> i32 {
    let (r, mut input) = parse_input(s).expect("valid input");
    assert!(r.is_empty());

    for instruction in input.instructions.clone() {
        // println!("ROBOT AT: {:?}", robot_pos);
        input.perform(instruction);
        // println!("ROBOT MOVED: {:?}", robot_pos);
        // println!("MAP:\n{}", input);
    }

    input
        .map
        .values_iter()
        .filter(|(_, value)| **value == Cell::Box)
        .map(|(p, _)| p.y * 100 + p.x)
        .sum()
}

pub fn part2(s: &str) -> i32 {
    // TODO: implement
    let (r, mut input) = parse_input(s).expect("valid input");
    assert!(r.is_empty());

    println!("MAP:\n{}", input);
    input = input.double_horizontally();

    println!("DOUBLED:");
    println!("MAP:\n{}", input);

    for instruction in input.instructions.clone() {
        // println!("ROBOT AT: {:?}", robot_pos);
        input.perform(instruction);
        // println!("ROBOT MOVED: {:?}", robot_pos);
        // println!("MAP:\n{}", input);
    }

    input
        .map
        .values_iter()
        .filter(|(_, value)| **value == Cell::LargeBoxLeft)
        .map(|(p, _)| p.y * 100 + p.x)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 2028);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
