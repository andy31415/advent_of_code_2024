use std::collections::{HashSet, VecDeque};

use glam::IVec2;
use map_parse::Parseable;
use nom::{character::complete::satisfy, Parser};

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Eq, Ord)]
enum Plant {
    Value(char),
}

impl Parseable for Plant {
    type Item = Plant;

    fn parse(s: &str) -> nom::IResult<&str, Self::Item> {
        satisfy(|c| c != '\r' && c != '\n')
            .map(Plant::Value)
            .parse(s)
    }
}

#[derive(Default, Debug, PartialEq, PartialOrd)]
struct GardenSize {
    cells: usize,
    walls: usize,
    long_walls: usize,
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn vec(&self) -> IVec2 {
        match self {
            Direction::N => IVec2::new(0, -1),
            Direction::E => IVec2::new(1, 0),
            Direction::S => IVec2::new(0, 1),
            Direction::W => IVec2::new(-1, 0),
        }
    }

    fn wall_neighbour(&self) -> Direction {
        match self {
            Direction::N | Direction::S => Direction::E,
            Direction::E | Direction::W => Direction::S,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum PlantType {
    Same,
    Different,
}

impl PlantType {
    fn same_plant(a: &Plant, b: &Plant) -> Self {
        if a == b {
            PlantType::Same
        } else {
            PlantType::Different
        }
    }
}

fn garden_size(
    start_position: &IVec2,
    garden: &map_parse::Map<Plant>,
    seen: &mut HashSet<IVec2>,
) -> GardenSize {
    if seen.contains(start_position) {
        return GardenSize::default();
    }

    let plant = garden
        .get(start_position)
        .expect("Starting position is valid");

    let mut to_check = VecDeque::new();

    seen.insert(*start_position);
    to_check.push_back(*start_position);

    let mut walls = 0;
    let mut cells = 0;
    let mut long_walls = 0;

    tracing::debug!("Checking {:?} i.e {:?}", start_position, plant);
    while let Some(value) = to_check.pop_front() {
        cells += 1;

        for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
            let new_pos = value + dir.vec();

            let other = garden.get(&new_pos);

            if other == Some(plant) {
                if !seen.contains(&new_pos) {
                    seen.insert(new_pos);
                    to_check.push_back(new_pos);
                }
            } else {
                walls += 1;

                tracing::debug!(
                    " Wall found between {:?} and {:?} (from {:?}",
                    value,
                    new_pos,
                    dir
                );

                let neighbour_pos = value + dir.wall_neighbour().vec();

                tracing::debug!("   Neighbour: {:?}", neighbour_pos);
                let n1 = garden
                    .get(&neighbour_pos)
                    .map(|x| PlantType::same_plant(x, plant))
                    .unwrap_or(PlantType::Different);
                let n2 = garden
                    .get(&(neighbour_pos + dir.vec()))
                    .map(|x| PlantType::same_plant(x, plant))
                    .unwrap_or(PlantType::Different);

                tracing::debug!(
                    "     Testing : {:?}/{:?} vs {:?}/{:?}",
                    plant,
                    other,
                    n1,
                    n2
                );

                if n1 != PlantType::Same || n2 != PlantType::Different {
                    tracing::debug!("      Long wall");
                    long_walls += 1;
                }
            }
        }
    }
    tracing::info!(
        "Starting at {:?} cell {:?} has {} walls, {} long_walls and {} cells",
        start_position,
        plant,
        walls,
        long_walls,
        cells
    );
    GardenSize {
        cells,
        walls,
        long_walls,
    }
}

pub fn part1(input: &str) -> usize {
    let (r, m) = map_parse::Map::<Plant>::parse(input).expect("Valid input");
    assert!(r.is_empty());

    let mut seen = HashSet::new();

    m.values_iter()
        .map(|(pos, _)| {
            let g = garden_size(pos, &m, &mut seen);
            g.walls * g.cells
        })
        .sum()
}

pub fn part2(input: &str) -> usize {
    let (r, m) = map_parse::Map::<Plant>::parse(input).expect("Valid input");
    assert!(r.is_empty());

    let mut seen = HashSet::new();

    m.values_iter()
        .map(|(pos, _)| {
            let g = garden_size(pos, &m, &mut seen);
            g.long_walls * g.cells
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1(include_str!("../example.txt")), 1930);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(include_str!("../example.txt")), 1206);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_part2_other() {
        assert_eq!(part2("AAAA\nBBCD\nBBCC\nEEEC"), 80);
    }
}
