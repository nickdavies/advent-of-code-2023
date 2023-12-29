use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;

use aoc_lib::grid::{Direction, Location, Map};

advent_of_code::solution!(16);

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

fn follow_path(
    map: &Map<Mirror>,
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
    let map = Map::try_from(input).context("failed to parse input")?;
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
    let map = Map::try_from(input).context("failed to parse input")?;

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
