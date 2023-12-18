use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::BinaryHeap;
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
    fn idx(&self) -> usize {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }
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

type Grid<T> = Vec<Vec<T>>;

#[derive(Debug)]
struct Map(Grid<usize>);

impl Map {
    fn get(&self, location: &Location) -> usize {
        self.0[location.0][location.1]
    }

    fn get_location(&self, x: usize, y: usize) -> Option<Location> {
        self.0
            .get(x)
            .and_then(|row| row.get(y))
            .map(|_| Location(x, y))
    }

    fn bottom_right(&self) -> Location {
        Location(self.0.len() - 1, self.0[self.0.len() - 1].len() - 1)
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

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
struct Location(usize, usize);

impl Location {
    fn manhattan_dist(&self, other: &Self) -> usize {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

#[derive(Clone, Debug)]
struct Movement<'a> {
    map: &'a Map,
    location: Location,
    direction: Direction,
    min_distance: usize,
    max_distance: usize,
    current_distance: usize,
    total_cost: usize,
    // We cache by the same vec shape as the input map
    // and also for each input direction.
    best: Rc<RefCell<Grid<[Option<usize>; 4]>>>,
}

impl PartialEq for Movement<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.weight() == other.weight()
    }
}
impl Eq for Movement<'_> {}

impl PartialOrd for Movement<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Movement<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let target = self.map.bottom_right();

        let my_weight = self.total_cost + self.location.manhattan_dist(&target);
        let other_weight = other.total_cost + other.location.manhattan_dist(&target);

        my_weight.cmp(&other_weight).reverse()
    }
}

impl<'a> Movement<'a> {
    fn new(
        map: &'a Map,
        location: Location,
        direction: Direction,
        min_distance: usize,
        max_distance: usize,
    ) -> Self {
        let mut cache = Vec::new();
        for row in &map.0 {
            let mut cache_row = Vec::new();
            for _ in row {
                cache_row.push([None; 4]);
            }
            cache.push(cache_row);
        }
        Self {
            map,
            location,
            direction,
            min_distance,
            max_distance,
            current_distance: 0,
            total_cost: 0,
            best: Rc::new(RefCell::new(cache)),
        }
    }

    fn weight(&self) -> usize {
        let target = self.map.bottom_right();
        self.total_cost + self.location.manhattan_dist(&target)
    }

    fn test_path(&self, new_direction: Direction) -> Option<Self> {
        if let Some(new_loc) = self.map.go_direction(&self.location, &new_direction) {
            let cost = self.map.get(&new_loc);
            let total_cost = self.total_cost + cost;
            let cache = &mut self.best.borrow_mut()[new_loc.0][new_loc.1][new_direction.idx()];
            match cache {
                Some(lowest_cost) => {
                    if &total_cost < lowest_cost {
                        *cache = Some(total_cost);
                    } else {
                        // We have been here with 0 distance, in the same direction with
                        // lower cost. There is no need to go this way again.
                        return None;
                    }
                }
                None => {
                    *cache = Some(total_cost);
                }
            }
            return Some(Self {
                map: self.map,
                location: new_loc,
                direction: new_direction,
                min_distance: self.min_distance,
                max_distance: self.max_distance,
                current_distance: 1,
                total_cost,
                best: self.best.clone(),
            });
        }
        None
    }

    fn available_paths(&self) -> Vec<Self> {
        let mut out = Vec::new();

        let can_turn = self.current_distance >= self.min_distance;

        if can_turn {
            let left = self.direction.left();
            if let Some(left_node) = self.test_path(left) {
                out.push(left_node);
            }

            let right = self.direction.right();
            if let Some(right_node) = self.test_path(right) {
                out.push(right_node);
            }
        }

        if self.current_distance < self.max_distance {
            if let Some(next_loc) = self.map.go_direction(&self.location, &self.direction) {
                let cost = self.map.get(&next_loc);
                out.push(Self {
                    map: self.map,
                    location: next_loc,
                    direction: self.direction.clone(),
                    min_distance: self.min_distance,
                    max_distance: self.max_distance,
                    current_distance: self.current_distance + 1,
                    total_cost: self.total_cost + cost,
                    best: self.best.clone(),
                })
            }
        }

        out
    }
}

fn seek_end(input: &str, min_distance: usize, max_distance: usize) -> Result<Option<usize>> {
    let map = parse_input(input).context("Failed to parse input")?;

    let mut to_visit = BinaryHeap::new();
    to_visit.push(Movement::new(
        &map,
        map.get_location(0, 0).context("Expected to find (0,0)")?,
        Direction::East,
        min_distance,
        max_distance,
    ));
    to_visit.push(Movement::new(
        &map,
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
        let node = to_visit.pop().unwrap();
        for next_node in node.available_paths() {
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
            to_visit.push(next_node);
        }
    }
    Ok(best.map(|n| n.total_cost))
}

fn parse_input(input: &str) -> Result<Map> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_line = Vec::new();
        for char in line.chars() {
            out_line.push(char.to_digit(10).context("Failed to parse input digit")? as usize);
        }
        out.push(out_line);
    }
    Ok(Map(out))
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    seek_end(input, 0, 3)
}

pub fn part_two(input: &str) -> Result<Option<usize>, anyhow::Error> {
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
