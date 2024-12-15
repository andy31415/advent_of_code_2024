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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Left,
    Down,
    Up,
    Right,
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
}

fn parse_input(s: &str) -> nom::IResult<&str, Input> {
    let (rest, map) = Map::<Cell>::parse(s)?;

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

    Ok((rest, Input { map, instructions }))
}

pub fn part1(s: &str) -> usize {
    let (r, input) = parse_input(s).expect("valid input");
    assert!(r.is_empty());

    // TODO: implement
    0
}

pub fn part2(input: &str) -> usize {
    // TODO: implement
    0
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
