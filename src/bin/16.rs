use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;

advent_of_code::solution!(16);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
enum Mirror {
    Empty,
    VertSplit,
    HozSplit,
    EastMirror,
    WestMirror,
}

impl Mirror {
    fn get_next(&self, inbound: &Direction) -> (Direction, Option<Direction>) {
        match (self, inbound) {
            (Self::Empty, _) => (inbound.clone(), None),
            (Self::VertSplit, _) => match inbound {
                Direction::North | Direction::South => (inbound.clone(), None),
                Direction::East | Direction::West => (Direction::North, Some(Direction::South)),
            },
            (Self::HozSplit, _) => match inbound {
                Direction::East | Direction::West => (inbound.clone(), None),
                Direction::North | Direction::South => (Direction::East, Some(Direction::West)),
            },
            // EastMirror is /
            (Self::EastMirror, inbound) => match inbound {
                Direction::North => (Direction::East, None),
                Direction::East => (Direction::North, None),
                Direction::South => (Direction::West, None),
                Direction::West => (Direction::South, None),
            },
            // WestMirror is \
            (Self::WestMirror, inbound) => match inbound {
                Direction::North => (Direction::West, None),
                Direction::East => (Direction::South, None),
                Direction::South => (Direction::East, None),
                Direction::West => (Direction::North, None),
            },
        }
    }
}

impl TryFrom<char> for Mirror {
    type Error = anyhow::Error;

    fn try_from(other: char) -> Result<Self, Self::Error> {
        match other {
            '.' => Ok(Self::Empty),
            '|' => Ok(Self::VertSplit),
            '-' => Ok(Self::HozSplit),
            '/' => Ok(Self::EastMirror),
            '\\' => Ok(Self::WestMirror),
            other => Err(anyhow!("Got unknown character {} in input", other)),
        }
    }
}

#[derive(Debug)]
struct Map(Vec<Vec<Mirror>>);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
struct Location(usize, usize);

impl Map {
    fn get(&self, location: &Location) -> &Mirror {
        &self.0[location.0][location.1]
    }

    fn get_location(&self, x: usize, y: usize) -> Option<Location> {
        self.0
            .get(x)
            .and_then(|row| row.get(y))
            .map(|_| Location(x, y))
    }

    fn go_direction(&self, current: &Location, direction: &Direction) -> Option<Location> {
        match direction {
            Direction::North => {
                if current.0 != 0 {
                    Some(Location(current.0 - 1, current.1))
                } else {
                    None
                }
            }
            Direction::East => self.get_location(current.0, current.1 + 1),
            Direction::South => self.get_location(current.0 + 1, current.1),
            Direction::West => {
                if current.1 != 0 {
                    Some(Location(current.0, current.1 - 1))
                } else {
                    None
                }
            }
        }
    }

    fn get_edges(&self) -> Vec<(Location, Direction)> {
        let mut out = Vec::new();

        for col in 0..self.0[0].len() {
            out.push((Location(0, col), Direction::South));
            out.push((Location(self.0.len() - 1, col), Direction::North));
        }
        for (row_num, row) in self.0.iter().enumerate() {
            out.push((Location(row_num, 0), Direction::East));
            out.push((Location(row_num, row.len() - 1), Direction::West));
        }

        out
    }
}

fn parse_input(input: &str) -> Result<Map> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_line = Vec::new();
        for char in line.chars() {
            out_line.push(char.try_into().context("Failed to parse character")?);
        }
        out.push(out_line);
    }

    Ok(Map(out))
}

fn follow_path(
    map: &Map,
    location: Location,
    direction: Direction,
    seen: &mut BTreeSet<(Location, Direction)>,
) -> Vec<(Location, Direction)> {
    let mut out = Vec::new();
    let key = (location.clone(), direction.clone());
    if seen.contains(&key) {
        return out;
    }
    seen.insert(key.clone());
    out.push(key);

    let (one_dir, two_dir) = map.get(&location).get_next(&direction);
    if let Some(one_loc) = map.go_direction(&location, &one_dir) {
        out.extend(follow_path(map, one_loc, one_dir, seen));
    }

    if let Some(two_dir) = two_dir {
        if let Some(two_loc) = map.go_direction(&location, &two_dir) {
            out.extend(follow_path(map, two_loc, two_dir, seen));
        }
    }

    out
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let map = parse_input(input).context("failed to parse input")?;
    let mut seen = BTreeSet::new();

    let path = follow_path(
        &map,
        map.get_location(0, 0).context("Failed to get (0, 0)")?,
        Direction::East,
        &mut seen,
    );

    let locations: BTreeSet<&Location> = path.iter().map(|(l, _)| l).collect();
    Ok(Some(locations.len()))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let map = parse_input(input).context("failed to parse input")?;

    let mut max = 0;
    for (location, direction) in map.get_edges() {
        let mut seen = BTreeSet::new();

        let path = follow_path(&map, location, direction, &mut seen);

        let locations: BTreeSet<&Location> = path.iter().map(|(l, _)| l).collect();
        max = std::cmp::max(max, locations.len());
    }
    Ok(Some(max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(46));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(51));
        Ok(())
    }
}
