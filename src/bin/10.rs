use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;
advent_of_code::solution!(10);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn all() -> [Direction; 4] {
        [Self::North, Self::East, Self::South, Self::West]
    }

    fn invert(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
struct Location(usize, usize);

impl Location {
    fn is_within<T>(&self, map: &[Vec<T>]) -> bool {
        map.get(self.0)
            .and_then(|row| row.get(self.1))
            .is_some()
    }

    fn bind<T>(self, bounding_map: &Vec<Vec<T>>) -> Result<BoundLocation<'_, T>> {
        if !self.is_within(bounding_map) {
            return Err(anyhow!("Location {:?} is out of bounds of map", self));
        }
        Ok(BoundLocation {
            location: self,
            bounding_map,
        })
    }
}

#[derive(Debug, Clone)]
struct BoundLocation<'a, T> {
    location: Location,
    bounding_map: &'a Vec<Vec<T>>,
}

impl<'a, T> Eq for BoundLocation<'a, T> {}
impl<'a, T> PartialEq for BoundLocation<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl<'a, T> PartialOrd for BoundLocation<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for BoundLocation<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.location.cmp(&other.location)
    }
}

impl<'a, T: std::fmt::Debug> BoundLocation<'a, T> {
    fn get_direction(&self, direction: &Direction) -> Option<Self> {
        match direction {
            Direction::North => self.north(),
            Direction::East => self.east(),
            Direction::South => self.south(),
            Direction::West => self.west(),
        }
    }

    fn north(&self) -> Option<Self> {
        if self.location.0 == 0 {
            None
        } else {
            Some(Self {
                location: Location(self.location.0 - 1, self.location.1),
                bounding_map: self.bounding_map,
            })
        }
    }

    fn south(&self) -> Option<Self> {
        if self.location.0 == self.bounding_map.len() - 1 {
            None
        } else {
            Some(Self {
                location: Location(self.location.0 + 1, self.location.1),
                bounding_map: self.bounding_map,
            })
        }
    }

    fn east(&self) -> Option<Self> {
        if self.location.1 == self.bounding_map[self.location.0].len() - 1 {
            None
        } else {
            Some(Self {
                location: Location(self.location.0, self.location.1 + 1),
                bounding_map: self.bounding_map,
            })
        }
    }

    fn west(&self) -> Option<Self> {
        if self.location.1 == 0 {
            None
        } else {
            Some(Self {
                location: Location(self.location.0, self.location.1 - 1),
                bounding_map: self.bounding_map,
            })
        }
    }
}

type PipeLocation<'a> = BoundLocation<'a, Option<PipeType>>;

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
    pipes: Vec<Vec<RawMapValue>>,
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
        Ok(RawPipeMap { start, pipes })
    }
}

impl RawPipeMap {
    fn get_pipe(&self, location: &BoundLocation<RawMapValue>) -> &RawMapValue {
        &self.pipes[location.location.0][location.location.1]
    }

    fn resolve_pipe_map(self) -> Result<(Location, PipeMap)> {
        let start = self
            .start
            .clone()
            .context("No starting point found in map")?;
        let bound_start = start
            .bind(&self.pipes)
            .context("Start is an invalid location")?;
        let start_pipe = self
            .resolve_start_pipe(&bound_start)
            .context("Failed to resolve start pipe type")?;

        let mut out = Vec::new();
        for row in &self.pipes {
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

        let map = PipeMap { pipes: out };
        Ok((bound_start.location, map))
    }

    fn resolve_start_pipe(&self, start: &BoundLocation<'_, RawMapValue>) -> Result<PipeType> {
        let mut matching = Vec::new();
        for direction in Direction::all() {
            if let Some(neighbour) = start.get_direction(&direction) {
                if let RawMapValue::Pipe(pipe_type) = self.get_pipe(&neighbour) {
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
    pipes: Vec<Vec<Option<PipeType>>>,
}

impl PipeMap {
    fn print(&self, inside: &BTreeSet<PipeLocation<'_>>, outside: &BTreeSet<PipeLocation<'_>>) {
        for (row_num, row) in self.pipes.iter().enumerate() {
            for (col_num, col) in row.iter().enumerate() {
                let loc = Location(row_num, col_num)
                    .bind(&self.pipes)
                    .expect("Failed to bind to self");
                if inside.contains(&loc) {
                    print!("I");
                } else if outside.contains(&loc) {
                    print!("O");
                } else {
                    print!("{}", col.as_ref().map(|c| c.to_char()).unwrap_or('.'));
                }
            }
            println!();
        }
    }

    fn all_locations(&self) -> Vec<Vec<PipeLocation<'_>>> {
        let mut out = Vec::new();
        for (row_num, row) in self.pipes.iter().enumerate() {
            let mut out_row = Vec::new();
            for col in 0..row.len() {
                out_row.push(
                    Location(row_num, col)
                        .bind(&self.pipes)
                        .expect("Failed to bind to self"),
                );
            }
            out.push(out_row)
        }

        out
    }

    fn get(&self, location: &PipeLocation<'_>) -> Option<&PipeType> {
        self.pipes[location.location.0][location.location.1].as_ref()
    }

    fn get_loop<'a>(&'a self, start: &'a PipeLocation<'a>) -> Result<PipeLoop<'a>> {
        let mut nodes = BTreeSet::new();
        nodes.insert(start.clone());

        let directions = self.get(start).unwrap().directions();
        let mut current = start.get_direction(&directions[0]).unwrap();
        while &current != start {
            nodes.insert(current.clone());

            let directions = self.get(&current).unwrap().directions();
            let one = current.get_direction(&directions[0]).unwrap();
            let two = current.get_direction(&directions[1]).unwrap();
            match (nodes.contains(&one), nodes.contains(&two)) {
                (true, false) => {
                    current = two;
                }
                (false, true) => {
                    current = one;
                }
                (false, false) => {
                    return Err(anyhow!("Found disjointed node at {:?}", current.location));
                }
                (true, true) => {
                    if &one == start || &two == start {
                        break;
                    } else {
                        return Err(anyhow!("Found loop node at {:?}", current.location));
                    }
                }
            }
        }

        Ok(PipeLoop {
            map: self,
            all_nodes: nodes,
        })
    }
}

#[derive(Debug)]
struct PipeLoop<'a> {
    map: &'a PipeMap,
    all_nodes: BTreeSet<PipeLocation<'a>>,
}

impl<'a> PipeLoop<'a> {
    fn get(&self, location: &PipeLocation<'_>) -> Option<&PipeType> {
        if self.all_nodes.contains(location) {
            self.map.get(location)
        } else {
            None
        }
    }

    fn loop_only_map(&self) -> PipeMap {
        let mut out = Vec::new();
        for (row_num, row) in self.map.pipes.iter().enumerate() {
            let mut out_row = Vec::new();
            for col in 0..row.len() {
                let loc = Location(row_num, col)
                    .bind(&self.map.pipes)
                    .expect("Failed to bind to self");
                if self.all_nodes.contains(&loc) {
                    out_row.push(self.map.pipes[row_num][col].clone());
                } else {
                    out_row.push(None)
                }
            }
            out.push(out_row);
        }

        PipeMap { pipes: out }
    }
}

pub fn part_one(input: &str) -> Result<Option<usize>> {
    let raw_map: RawPipeMap = input.parse().context("Failed to parse map")?;
    let (start, map) = raw_map
        .resolve_pipe_map()
        .context("Failed to resolve pipe map")?;
    let start = start
        .bind(&map.pipes)
        .context("Failed to rebind start new map")?;

    let pipe_loop = map.get_loop(&start)?;
    Ok(Some(pipe_loop.all_nodes.len() / 2))
}

pub fn part_two(input: &str) -> Result<Option<usize>> {
    let mut out = 0;
    let raw_map: RawPipeMap = input.parse().context("Failed to parse map")?;
    let (start, map) = raw_map
        .resolve_pipe_map()
        .context("Failed to resolve pipe map")?;
    let start = start
        .bind(&map.pipes)
        .context("Failed to rebind start new map")?;

    let pipe_loop = map.get_loop(&start)?;

    pipe_loop
        .loop_only_map()
        .print(&BTreeSet::new(), &BTreeSet::new());

    let mut inside_nodes = BTreeSet::new();
    let mut outside_nodes = BTreeSet::new();

    for row in map.all_locations() {
        let mut inside = false;
        let mut elbow = None;
        for col in row {
            if let Some(pipe_type) = pipe_loop.get(&col) {
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
                inside_nodes.insert(col.clone());
            } else {
                outside_nodes.insert(col.clone());
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
        let result = part_one(input)?;
        assert_eq!(result, Some(4));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(10));
        Ok(())
    }
}
