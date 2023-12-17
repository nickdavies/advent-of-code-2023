use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

advent_of_code::solution!(17);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn left(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    fn right(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

#[derive(Debug)]
struct Map(Vec<Vec<u32>>);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
struct Location(usize, usize);

impl Map {
    fn get(&self, location: &Location) -> &u32 {
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
}

#[derive(Clone, Debug)]
struct Movement {
    location: Location,
    direction: Direction,
    min_distance: usize,
    max_distance: usize,
    current_distance: usize,
    total_cost: u32,
    //seen: Vec<Location>,
    best: Rc<RefCell<HashMap<(Location, Direction), u32>>>,
}

impl Movement {
    fn new(
        location: Location,
        direction: Direction,
        min_distance: usize,
        max_distance: usize,
    ) -> Self {
        Self {
            location,
            direction,
            min_distance,
            max_distance,
            current_distance: 0,
            total_cost: 0,
            //seen: Vec::new(),
            best: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn test_path(&self, map: &Map, new_direction: Direction) -> Option<Self> {
        if let Some(new_loc) = map.go_direction(&self.location, &new_direction) {
            let cost = map.get(&new_loc);
            let total_cost = self.total_cost + cost;
            let mut best = self.best.borrow_mut();
            let key = (new_loc, new_direction);
            match best.get(&key) {
                Some(lowest_cost) => {
                    if &total_cost < lowest_cost {
                        best.insert(key.clone(), total_cost);
                    } else {
                        // We have been here with 0 distance, in the same direction with
                        // lower cost. There is no need to go this way again.
                        return None;
                    }
                }
                None => {
                    best.insert(key.clone(), total_cost);
                }
            }

            let (new_loc, new_direction) = key;
            // let mut seen = self.seen.clone();
            // seen.push(new_loc.clone());
            return Some(Self {
                location: new_loc,
                direction: new_direction,
                min_distance: self.min_distance,
                max_distance: self.max_distance,
                current_distance: 1,
                total_cost,
                // seen,
                best: self.best.clone(),
            });
        }
        None
    }

    fn available_paths(&self, map: &Map) -> Vec<Self> {
        let mut out = Vec::new();

        if self.current_distance >= self.min_distance {
            let left = self.direction.left();
            if let Some(left_node) = self.test_path(map, left) {
                out.push(left_node);
            }

            let right = self.direction.right();
            if let Some(right_node) = self.test_path(map, right) {
                out.push(right_node);
            }
        }

        if self.current_distance < self.max_distance {
            if let Some(next_loc) = map.go_direction(&self.location, &self.direction) {
                // let mut seen = self.seen.clone();
                // seen.push(next_loc.clone());
                let cost = map.get(&next_loc);
                out.push(Self {
                    location: next_loc,
                    direction: self.direction.clone(),
                    min_distance: self.min_distance,
                    max_distance: self.max_distance,
                    current_distance: self.current_distance + 1,
                    total_cost: self.total_cost + cost,
                    // seen,
                    best: self.best.clone(),
                })
            }
        }

        out
    }
}

fn seek_end(input: &str, min_distance: usize, max_distance: usize) -> Result<Option<u32>> {
    let map = parse_input(input).context("Failed to parse input")?;

    let mut costs: Vec<Vec<Option<u32>>> = Vec::new();
    for row in &map.0 {
        costs.push(vec![None; row.len()]);
    }

    let mut to_visit = VecDeque::new();
    to_visit.push_front(Movement::new(
        map.get_location(0, 0).context("Expected to find (0,0)")?,
        Direction::East,
        min_distance,
        max_distance,
    ));
    to_visit.push_front(Movement::new(
        map.get_location(0, 0).context("Expected to find (0,0)")?,
        Direction::South,
        min_distance,
        max_distance,
    ));

    let target = map
        .get_location(map.0.len() - 1, map.0[map.0.len() - 1].len() - 1)
        .context("Expected to find bottom right")?;
    let mut best: Option<Movement> = None;
    while !to_visit.is_empty() {
        let node = to_visit.pop_front().unwrap();
        for next_node in node.available_paths(&map) {
            if next_node.location == target && next_node.current_distance >= min_distance {
                match &best {
                    Some(best_cost) => {
                        if next_node.total_cost < best_cost.total_cost {
                            best = Some(next_node.clone());
                        }
                    }
                    None => {
                        best = Some(next_node.clone());
                    }
                }
            }
            // println!(
            //     "HERE: {:?} -> {:?} ({}/{})",
            //     node.location,
            //     next_node.location,
            //     next_node.seen.len(),
            //     next_node.total_cost
            // );
            to_visit.push_back(next_node);
        }
    }

    // if let Some(best) = &best {
    //     println!("{:?}, {:?}", best.seen, target);
    // };

    Ok(best.map(|n| n.total_cost))
}

fn parse_input(input: &str) -> Result<Map> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_line = Vec::new();
        for char in line.chars() {
            out_line.push(char.to_digit(10).context("Failed to parse input digit")?);
        }
        out.push(out_line);
    }
    Ok(Map(out))
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    seek_end(input, 0, 3)
}

pub fn part_two(input: &str) -> Result<Option<u32>, anyhow::Error> {
    seek_end(input, 4, 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(102));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(94));
        Ok(())
    }

    #[test]
    fn test_part_two_example_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 3);
        let result = part_two(input)?;
        assert_eq!(result, Some(71));
        Ok(())
    }
}
