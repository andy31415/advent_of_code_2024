use nom::{
    bytes::complete::tag,
    character::complete::{self, line_ending},
    combinator::opt,
    multi::separated_list1,
    sequence::tuple,
    IResult, Parser,
};
use nom_supreme::ParserExt;

#[derive(Debug, PartialEq, Eq)]
struct Values {
    x: i64,
    y: i64,
}

#[derive(Debug)]
struct ClawMachine {
    a: Values,
    b: Values,
    prize: Values,
}

impl ClawMachine {
    fn compute_presses(&self) -> (i64, i64) {
        let b = (self.prize.x * self.a.y - self.prize.y * self.a.x)
            / (self.a.y * self.b.x - self.a.x * self.b.y);

        let a = (self.prize.x * self.b.y - self.prize.y * self.b.x)
            / (self.b.y * self.a.x - self.b.x * self.a.y);

        if a * self.a.x + b * self.b.x != self.prize.x {
            tracing::info!("NO solution for {:?} - no X", self);
            return (0, 0);
        }

        if a * self.a.y + b * self.b.y != self.prize.y {
            tracing::info!("NO solution for {:?} - no Y", self);
            return (0, 0);
        }

        tracing::info!("B for {:?} is {} and {}", self, a, b);

        (a, b)
    }
}

fn parse_claw_machine(input: &str) -> IResult<&str, ClawMachine> {
    tuple((
        tuple((
            complete::i64.preceded_by(tag("Button A: X+")),
            complete::i64.preceded_by(tag(", Y+")),
        ))
        .terminated(line_ending),
        tuple((
            complete::i64.preceded_by(tag("Button B: X+")),
            complete::i64.preceded_by(tag(", Y+")),
        ))
        .terminated(line_ending),
        tuple((
            complete::i64.preceded_by(tag("Prize: X=")),
            complete::i64.preceded_by(tag(", Y=")),
        ))
        .terminated(opt(line_ending)),
    ))
    .map(|((ax, ay), (bx, by), (px, py))| ClawMachine {
        a: Values { x: ax, y: ay },
        b: Values { x: bx, y: by },
        prize: Values { x: px, y: py },
    })
    .parse(input)
}

fn parse_machines(input: &str) -> IResult<&str, Vec<ClawMachine>> {
    separated_list1(line_ending, parse_claw_machine).parse(input)
}

pub fn part1(input: &str) -> i64 {
    let (r, machines) = parse_machines(input).expect("valid input");
    assert!(r.is_empty());

    machines
        .iter()
        .map(|m| {
            let (a, b) = m.compute_presses();
            a * 3 + b
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
    #[tracing_test::traced_test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 480);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
