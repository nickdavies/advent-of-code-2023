use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;

use aoc_lib::grid::{Direction, Location, Map};

advent_of_code::solution!(10);

#[derive(Debug, Clone, PartialEq)]
enum PipeType {
    Vertical,
    Horizontal,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl PipeType {
    fn to_char(&self) -> char {
        match self {
            Self::Vertical => '│',
            Self::Horizontal => '─',
            Self::NorthEast => '└',
            Self::NorthWest => '┘',
            Self::SouthEast => '┌',
            Self::SouthWest => '┐',
        }
    }

    fn directions(&self) -> [Direction; 2] {
        match self {
            Self::Vertical => [Direction::North, Direction::South],
            Self::Horizontal => [Direction::East, Direction::West],
            Self::NorthEast => [Direction::North, Direction::East],
            Self::NorthWest => [Direction::North, Direction::West],
            Self::SouthEast => [Direction::South, Direction::East],
            Self::SouthWest => [Direction::South, Direction::West],
        }
    }

    fn has_direction(&self, direction: &Direction) -> bool {
        self.directions().contains(direction)
    }
}

impl TryFrom<char> for PipeType {
    type Error = anyhow::Error;

    fn try_from(input: char) -> Result<Self, Self::Error> {
        Ok(match input {
            '|' => Self::Vertical,
            '-' => Self::Horizontal,
            'L' => Self::NorthEast,
            'J' => Self::NorthWest,
            '7' => Self::SouthWest,
            'F' => Self::SouthEast,
            other => {
                return Err(anyhow!("Found unexpected char '{}'", other));
            }
        })
    }
}

#[derive(Debug, PartialEq)]
enum RawMapValue {
    Start,
    Empty,
    Pipe(PipeType),
}

impl TryFrom<char> for RawMapValue {
    type Error = anyhow::Error;

    fn try_from(input: char) -> Result<Self, Self::Error> {
        Ok(match input {
            'S' => Self::Start,
            '.' => Self::Empty,
            other => Self::Pipe(other.try_into()?),
        })
    }
}

#[derive(Debug)]
struct RawPipeMap {
    start: Option<Location>,
    pipes: Map<RawMapValue>,
}

impl std::str::FromStr for RawPipeMap {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut pipes = Vec::new();
        let mut start = None;
        for (row_num, line) in input.lines().enumerate() {
            let mut row = Vec::new();
            for (col_num, char) in line.chars().enumerate() {
                let value = char
                    .try_into()
                    .context(format!("Failed to parse row {} col {}", row_num, col_num))?;

                if value == RawMapValue::Start {
                    start = Some(Location(row_num, col_num));
                }
                row.push(value);
            }
            pipes.push(row);
        }
        Ok(RawPipeMap {
            start,
            pipes: Map(pipes),
        })
    }
}

impl RawPipeMap {
    fn resolve_pipe_map(self) -> Result<(Location, PipeMap)> {
        let start = self
            .start
            .clone()
            .context("No starting point found in map")?;
        let start_pipe = self
            .resolve_start_pipe(&start)
            .context("Failed to resolve start pipe type")?;

        let mut out = Vec::new();
        for row in &self.pipes.0 {
            let mut out_row = Vec::new();
            for value in row {
                out_row.push(match value {
                    RawMapValue::Pipe(pipe_type) => Some(pipe_type.clone()),
                    RawMapValue::Start => Some(start_pipe.clone()),
                    RawMapValue::Empty => None,
                });
            }
            out.push(out_row);
        }

        let map = PipeMap { pipes: Map(out) };
        Ok((start, map))
    }

    fn resolve_start_pipe(&self, start: &Location) -> Result<PipeType> {
        let mut matching = Vec::new();
        for direction in Direction::all() {
            if let Some(neighbour) = self.pipes.go_direction(start, direction) {
                if let RawMapValue::Pipe(pipe_type) = self.pipes.get(&neighbour) {
                    let inv = direction.invert();
                    if pipe_type.has_direction(&inv) {
                        matching.push(direction);
                    }
                }
            }
        }

        let mut matching = matching.into_iter();
        let first = matching
            .next()
            .context("Expected to find 2 matching directions found 0")?;
        let second = matching
            .next()
            .context("Expected to find 2 matching directions found 0")?;

        let remaining = matching.count();
        if remaining != 0 {
            return Err(anyhow!(
                "Expected exactly 2 matching pipe directions found {}",
                2 + remaining
            ));
        }

        Ok(match (&first, &second) {
            (Direction::North, Direction::South) | (Direction::South, Direction::North) => {
                PipeType::Vertical
            }
            (Direction::East, Direction::West) | (Direction::West, Direction::East) => {
                PipeType::Horizontal
            }
            (Direction::North, Direction::East) | (Direction::East, Direction::North) => {
                PipeType::NorthEast
            }
            (Direction::North, Direction::West) | (Direction::West, Direction::North) => {
                PipeType::NorthWest
            }
            (Direction::South, Direction::East) | (Direction::East, Direction::South) => {
                PipeType::SouthEast
            }
            (Direction::South, Direction::West) | (Direction::West, Direction::South) => {
                PipeType::SouthWest
            }
            (Direction::North, Direction::North)
            | (Direction::East, Direction::East)
            | (Direction::South, Direction::South)
            | (Direction::West, Direction::West) => {
                return Err(anyhow!(
                    "Found invalid pipe pattern: {:?} {:?}",
                    first,
                    second
                ));
            }
        })
    }
}

#[derive(Debug)]
struct PipeMap {
    pipes: Map<Option<PipeType>>,
}

impl PipeMap {
    fn print(&self, inside: &BTreeSet<Location>, outside: &BTreeSet<Location>) {
        self.pipes.print(|c, loc| {
            if inside.contains(&loc) {
                'I'
            } else if outside.contains(&loc) {
                'O'
            } else {
                c.as_ref().map(|c| c.to_char()).unwrap_or('.')
            }
        })
    }

    fn get_loop<'a>(&'a self, start: &'a Location) -> Result<PipeLoop<'a>> {
        let mut nodes = BTreeSet::new();
        nodes.insert(start.clone());

        let directions = self.pipes.get(start).as_ref().unwrap().directions();
        let mut current = self.pipes.go_direction(start, &directions[0]).unwrap();
        while &current != start {
            nodes.insert(current.clone());

            let directions = self.pipes.get(&current).as_ref().unwrap().directions();
            let one = self.pipes.go_direction(&current, &directions[0]).unwrap();
            let two = self.pipes.go_direction(&current, &directions[1]).unwrap();
            match (nodes.contains(&one), nodes.contains(&two)) {
                (true, false) => {
                    current = two;
                }
                (false, true) => {
                    current = one;
                }
                (false, false) => {
                    return Err(anyhow!("Found disjointed node at {:?}", current));
                }
                (true, true) => {
                    if &one == start || &two == start {
                        break;
                    } else {
                        return Err(anyhow!("Found loop node at {:?}", current));
                    }
                }
            }
        }

        Ok(PipeLoop {
            map: &self.pipes,
            all_nodes: nodes,
        })
    }
}

#[derive(Debug)]
struct PipeLoop<'a> {
    map: &'a Map<Option<PipeType>>,
    all_nodes: BTreeSet<Location>,
}

impl<'a> PipeLoop<'a> {
    fn get(&self, location: &Location) -> Option<&PipeType> {
        if self.all_nodes.contains(location) {
            self.map.get(location).as_ref()
        } else {
            None
        }
    }

    fn loop_only_map(&self) -> PipeMap {
        let new_map = self.map.transform(|loc, col| {
            if self.all_nodes.contains(&loc) {
                col.clone()
            } else {
                None
            }
        });
        PipeMap { pipes: new_map }
    }
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>> {
    let raw_map: RawPipeMap = input.parse().context("Failed to parse map")?;
    let (start, map) = raw_map
        .resolve_pipe_map()
        .context("Failed to resolve pipe map")?;

    let pipe_loop = map.get_loop(&start)?;
    Ok(Some(pipe_loop.all_nodes.len() / 2))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<usize>> {
    let mut out = 0;
    let raw_map: RawPipeMap = input.parse().context("Failed to parse map")?;
    let (start, map) = raw_map
        .resolve_pipe_map()
        .context("Failed to resolve pipe map")?;

    let pipe_loop = map.get_loop(&start)?;

    pipe_loop
        .loop_only_map()
        .print(&BTreeSet::new(), &BTreeSet::new());

    let mut inside_nodes = BTreeSet::new();
    let mut outside_nodes = BTreeSet::new();

    for row in map.pipes.iter() {
        let mut inside = false;
        let mut elbow = None;
        for (loc, _) in row {
            if let Some(pipe_type) = pipe_loop.get(&loc) {
                match pipe_type {
                    PipeType::Horizontal => continue,
                    PipeType::Vertical => {
                        inside = !inside;
                    }
                    PipeType::NorthEast => {
                        inside = !inside;
                        elbow = Some(pipe_type);
                    }
                    PipeType::SouthEast => {
                        inside = !inside;
                        elbow = Some(pipe_type);
                    }
                    PipeType::NorthWest => {
                        if elbow != Some(&PipeType::SouthEast) {
                            inside = !inside;
                        }
                        elbow = Some(pipe_type);
                    }
                    PipeType::SouthWest => {
                        if elbow != Some(&PipeType::NorthEast) {
                            inside = !inside;
                        }
                        elbow = Some(pipe_type);
                    }
                }
            } else if inside {
                out += 1;
                inside_nodes.insert(loc);
            } else {
                outside_nodes.insert(loc);
            }
        }
    }
    pipe_loop
        .loop_only_map()
        .print(&inside_nodes, &outside_nodes);

    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(4));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(10));
        Ok(())
    }
}
