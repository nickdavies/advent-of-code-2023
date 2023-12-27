use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;
use std::collections::BinaryHeap;

advent_of_code::solution!(21);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn all() -> &'static [Direction; 4] {
        &[
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
}

type Grid<T> = Vec<Vec<T>>;

#[derive(Debug)]
struct Map<T>(Grid<T>);

impl<T> Map<T> {
    fn get(&self, location: &Location) -> &T {
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

    fn bottom_right(&self) -> Option<Location> {
        let row = self.0.last()?;
        Some(Location(self.0.len() - 1, row.len() - 1))
    }

    fn values(&self) -> Vec<(Location, &T)> {
        let mut out = Vec::new();
        for (i, row) in self.0.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                out.push((Location(i, j), col))
            }
        }
        out
    }
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
struct Location(usize, usize);

fn parse_input<T>(input: &str, parse: fn(char) -> Result<(bool, T)>) -> Result<(Map<T>, Location)> {
    let mut out = Vec::new();
    let mut start = None;
    for (line_num, line) in input.lines().enumerate() {
        let mut out_line = Vec::new();
        for (char_num, char) in line.chars().enumerate() {
            let (maybe_start, value) = parse(char)?;
            if maybe_start {
                start = Some(Location(line_num, char_num));
            }
            out_line.push(value);
        }
        out.push(out_line);
    }
    Ok((Map(out), start.context("Expected to find starting square")?))
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
struct StartDelta {
    distance: usize,
    location: Location,
}

fn get_distances(map: &Map<bool>, start: Location, step_limit: usize) -> Map<Option<usize>> {
    let mut distances = Map(Vec::with_capacity(map.0.len()));
    for row in &map.0 {
        distances.0.push(vec![None; row.len()]);
    }

    let mut to_visit = BinaryHeap::new();
    to_visit.push(std::cmp::Reverse(StartDelta {
        location: start.clone(),
        distance: 0,
    }));

    while !to_visit.is_empty() {
        let loc = to_visit.pop().unwrap().0;

        let target = &mut distances.0[loc.location.0][loc.location.1];
        if target.is_some() || loc.distance > step_limit {
            continue;
        }
        *target = Some(loc.distance);

        for direction in Direction::all() {
            if let Some(next) = map.go_direction(&loc.location, direction) {
                if *map.get(&next) {
                    to_visit.push(std::cmp::Reverse(StartDelta {
                        location: next,
                        distance: loc.distance + 1,
                    }));
                }
            }
        }
    }
    distances
}

fn get_possible(grid: &Map<bool>, start_location: Location, steps: usize) -> BTreeSet<Location> {
    let distances = get_distances(grid, start_location, steps);
    let mut distances: Vec<(Location, usize)> = distances
        .values()
        .into_iter()
        .filter_map(|(l, v)| Some((l, (*v)?)))
        .collect();
    distances.sort_by_key(|(_, d)| *d);
    let mut out = BTreeSet::new();
    for (location, distance) in distances.iter() {
        if distance <= &steps && (&steps % 2) == distance % 2 {
            out.insert(location.clone());
        }
    }

    out
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let (grid, start_location) = parse_input(input, |char| match char {
        '.' => Ok((false, true)),
        '#' => Ok((false, false)),
        'S' => Ok((true, true)),
        other => Err(anyhow!("Unknown character {} input", other)),
    })
    .context("Failed to parse input")?;

    let options = get_possible(&grid, start_location, 64);
    Ok(Some(options.len()))
}

fn get_odd_even_counts(distances: &Map<Option<usize>>) -> (usize, usize) {
    let mut even = 0;
    let mut odd = 0;
    for (_, distance) in distances.values() {
        if let Some(distance) = distance {
            if distance % 2 == 0 {
                even += 1;
            } else {
                odd += 1;
            }
        }
    }
    (even, odd)
}

fn get_grid_sum(grid: &Map<bool>, start: Location, steps: usize) -> u64 {
    let tile_reach = (steps / grid.0.len()) as u64;
    println!("tile: {}", tile_reach);
    let mut odd_tiles: u64 = 1;
    let mut even_tiles: u64 = 0;
    for tile in 0..tile_reach {
        if tile % 2 == 1 {
            even_tiles += tile * 4;
        } else {
            odd_tiles += tile * 4;
        }
    }

    let (even, odd) = get_odd_even_counts(&get_distances(grid, start, steps));
    (odd_tiles * odd as u64) + (even_tiles * even as u64)
}

fn get_centered_sum(grid: &Map<bool>, start: Location, steps: usize) -> u64 {
    let br = grid.bottom_right().unwrap();
    let locations = vec![
        Location(start.0, 0),    // From North
        Location(br.0, start.1), // From East
        Location(start.0, br.1), // From South
        Location(0, start.1),    // From West
    ];

    let mut sum: u64 = 0;
    let step_limit = (steps - start.0 - 1) % grid.0.len();
    for location in locations {
        let (even, odd) = get_odd_even_counts(&get_distances(grid, location, step_limit));
        if step_limit % 2 == 0 {
            sum += even as u64;
        } else {
            sum += odd as u64;
        }
    }

    sum
}

fn get_diag_sum(grid: &Map<bool>, start: Location, steps: usize) -> u64 {
    let br = grid.bottom_right().unwrap();
    let locations = vec![
        Location(0, 0),       // From NW
        Location(br.0, 0),    // From SW
        Location(br.0, br.1), // From SE
        Location(0, br.1),    // From NE
    ];

    let h = grid.0.len();
    let w = grid.0[0].len();
    println!("h={}, w={}", h, w);
    let tile_reach = (steps / h) as u64;
    let lower_step = (steps - start.0 - start.1 - h - 2) % (w + h);
    let upper_step = (steps - start.0 - start.1 - 2) % (w + h);

    println!("tile={}", tile_reach);
    println!("upper={} lower={}", upper_step, lower_step);
    let mut sum: u64 = 0;
    for location in locations {
        let (lo_even, lo_odd) =
            get_odd_even_counts(&get_distances(grid, location.clone(), lower_step));
        let (hi_even, hi_odd) = get_odd_even_counts(&get_distances(grid, location, upper_step));
        if lower_step % 2 == 0 {
            sum += lo_even as u64 * tile_reach;
            sum += hi_odd as u64 * (tile_reach - 1);
        } else {
            sum += lo_odd as u64 * tile_reach;
            sum += hi_even as u64 * (tile_reach - 1);
        }
    }

    sum
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let (grid, start_location) = parse_input(input, |char| match char {
        '.' => Ok((false, true)),
        '#' => Ok((false, false)),
        'S' => Ok((true, true)),
        other => Err(anyhow!("Unknown character {} input", other)),
    })
    .context("Failed to parse input")?;

    // logic mostly stolen from:
    // https://github.com/NickLanam/advent-of-code/blob/main/2023/day21.mjs
    let steps = 26501365;
    let grid_sum = get_grid_sum(&grid, start_location.clone(), steps);
    println!("grid={}", grid_sum);
    let center_sum = get_centered_sum(&grid, start_location.clone(), steps);
    println!("center={}", center_sum);
    let diag_sum = get_diag_sum(&grid, start_location, steps);
    println!("diag={}", diag_sum);

    Ok(Some(grid_sum + center_sum + diag_sum))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(42));
        Ok(())
    }

    #[test]
    fn test_part_one_nodes() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let answer = &advent_of_code::template::read_file_part("examples", DAY, 3);
        let (answer_grid, _) = parse_input(answer, |char| match char {
            'O' => Ok((true, Some(true))),
            '.' => Ok((true, Some(false))),
            '#' => Ok((true, None)),
            other => {
                return Err(anyhow!("Unknown character {} input", other));
            }
        })?;

        let (grid, start_location) = parse_input(input, |char| match char {
            '.' => Ok((false, true)),
            '#' => Ok((false, false)),
            'S' => Ok((true, true)),
            other => {
                return Err(anyhow!("Unknown character {} input", other));
            }
        })
        .context("Failed to parse input")?;

        let options = get_possible(&grid, start_location, 6);

        let mut matching = true;
        for (i, row) in answer_grid.0.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                let loc = Location(i, j);
                if col.unwrap_or(false) != options.contains(&loc) {
                    matching = false;
                    println!("Mismatch {:?}: {:?} {}", loc, col, options.contains(&loc));
                }
            }
        }

        assert!(matching);
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(598044246091826));
        Ok(())
    }
}
