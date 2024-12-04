#[derive(PartialEq, Eq, Debug)]
struct Multiplication {
    left: usize,
    right: usize,
}

impl Multiplication {
    fn value(&self) -> usize {
        self.left * self.right
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Input {
    items: Vec<Multiplication>,
}

mod parse {

    use super::{Input, Multiplication};
    use regex::Regex;

    pub(crate) fn input(s: &str) -> Input {
        let expr = Regex::new(r"mul\((?<left>\d+),(?<right>\d+)\)").expect("Valid expression");

        Input {
            items: expr
                .captures_iter(s)
                .map(|m| Multiplication {
                    left: m
                        .name("left")
                        .expect("Has a match")
                        .as_str()
                        .parse()
                        .expect("valid"),
                    right: m
                        .name("right")
                        .expect("Has a match")
                        .as_str()
                        .parse()
                        .expect("valid"),
                })
                .collect(),
        }
    }
}

pub fn part1(s: &str) -> usize {
    parse::input(s)
        .items
        .iter()
        .map(Multiplication::value)
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

pub fn part2(_s: &str) -> usize {
    // TODO: implement
    0
}

#[cfg(test)]
mod tests {
    use super::parse::*;
    use super::*;

    #[test]
    fn test_input_parse() {
        assert_eq!(input(""), Input { items: vec![] });
        assert_eq!(input("test"), Input { items: vec![] });
        assert_eq!(input("mul(1,2"), Input { items: vec![] });
        assert_eq!(
            input("mul(1,2)"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("mul(1,2)foo"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("foomul(1,2)bar"),
            Input {
                items: vec![Multiplication { left: 1, right: 2 }]
            }
        );
        assert_eq!(
            input("foomul(1,2)barmul(3,4)"),
            Input {
                items: vec![
                    Multiplication { left: 1, right: 2 },
                    Multiplication { left: 3, right: 4 },
                ]
            }
        );
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 161);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 0);
    }
}
