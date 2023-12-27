use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use petgraph::algo::simple_paths::all_simple_paths;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use petgraph::{Directed, EdgeType, Undirected};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

advent_of_code::solution!(23);

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

impl Map<MapValue> {
    fn adjacent(&self, location: Location) -> Vec<(Direction, Location)> {
        Direction::all()
            .iter()
            .filter_map(|direction| {
                let next_location = self.go_direction(&location, direction)?;
                if let MapValue::Forest = self.get(&next_location) {
                    None
                } else {
                    Some((direction.clone(), next_location))
                }
            })
            .collect()
    }

    fn find_junctions(&self) -> BTreeSet<Location> {
        let mut junctions = BTreeSet::new();
        for (location, value) in self.values() {
            if let MapValue::Path = value {
            } else {
                continue;
            }

            if self.adjacent(location).len() >= 3 {
                junctions.insert(location);
            }
        }

        junctions
    }

    fn seek_from(
        &self,
        from: Location,
        key_locations: &BTreeSet<Location>,
        climb_slopes: bool,
    ) -> Vec<(Location, usize)> {
        let mut out = Vec::new();

        let mut to_visit = VecDeque::new();
        to_visit.push_back(NodeVisit {
            location: from,
            prev: None,
            distance: 0,
        });

        let mut seen = BTreeSet::new();
        while !to_visit.is_empty() {
            let current = to_visit.pop_front().unwrap();
            if seen.contains(&current) {
                continue;
            }
            seen.insert(current.clone());

            for (next_direction, next) in self.adjacent(current.location) {
                if let Some(prev) = current.prev {
                    if prev == next {
                        continue;
                    }
                }
                // If we reached a a key node add it and don't continue
                if key_locations.contains(&next) {
                    out.push((next, current.distance + 1));
                // If we don't have a key node then we
                } else {
                    let add = match self.get(&next) {
                        // We always go down paths and assume that we haven't seen it before
                        MapValue::Path => true,
                        // For slops we must only approach them for their direction
                        MapValue::Slope(slope_direction) => {
                            climb_slopes || (&next_direction == slope_direction)
                        }
                        MapValue::Forest => false,
                    };
                    if add {
                        to_visit.push_back(NodeVisit {
                            location: next,
                            prev: Some(current.location),
                            distance: current.distance + 1,
                        });
                    }
                }
            }
        }
        out
    }

    fn build_graph<D: EdgeType>(
        &self,
        start: Location,
        end: Location,
        climb_slopes: bool,
    ) -> Result<(Graph<Location, usize, D>, NodeIndex, NodeIndex)> {
        let mut junctions = self.find_junctions();
        junctions.insert(start);
        junctions.insert(end);

        let mut out =
            Graph::<Location, usize, D>::with_capacity(junctions.len() + 2, junctions.len() * 3);
        let mut node_map = BTreeMap::new();
        for junction in &junctions {
            let node_id = out.add_node(*junction);
            node_map.insert(*junction, node_id);
        }

        for junction in &junctions {
            let key_nodes = self.seek_from(*junction, &junctions, climb_slopes);
            for (target, distance) in key_nodes {
                out.add_edge(
                    *node_map.get(junction).unwrap(),
                    *node_map.get(&target).unwrap(),
                    distance,
                );
            }
        }
        Ok((
            out,
            *node_map.get(&start).unwrap(),
            *node_map.get(&end).unwrap(),
        ))
    }

    fn longest_path<E: EdgeType>(
        &self,
        start: Location,
        end: Location,
        climb_slopes: bool,
    ) -> Result<Option<usize>> {
        let (graph, start_node, end_node) = self
            .build_graph::<E>(start, end, climb_slopes)
            .context("Failed to make graph from grid")?;

        let longest = all_simple_paths::<Vec<_>, _>(&graph, start_node, end_node, 0, None)
            .map(|p| {
                p.windows(2)
                    .map(|w| graph.edges_connecting(w[0], w[1]).next().unwrap().weight())
                    .sum()
            })
            .max();
        Ok(longest)
    }
}

#[derive(Debug, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
struct Location(usize, usize);

#[derive(Debug)]
enum MapValue {
    Path,
    Forest,
    Slope(Direction),
}

fn find_single_path(row: &[MapValue]) -> Result<usize> {
    let mut target = None;
    for (i, value) in row.iter().enumerate() {
        if let MapValue::Path = value {
            if target.is_some() {
                return Err(anyhow!("Found at least two path blocks in row"));
            }
            target = Some(i);
        }
    }
    target.context("Expected to find exactly 1 path square, found 0")
}

fn parse_input(input: &str) -> Result<(Map<MapValue>, Location, Location)> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_line = Vec::new();
        for char in line.chars() {
            match char {
                '#' => out_line.push(MapValue::Forest),
                '.' => out_line.push(MapValue::Path),
                '^' => out_line.push(MapValue::Slope(Direction::North)),
                '>' => out_line.push(MapValue::Slope(Direction::East)),
                'v' => out_line.push(MapValue::Slope(Direction::South)),
                '<' => out_line.push(MapValue::Slope(Direction::West)),
                other => {
                    return Err(anyhow!("Got unexpected value {} in input", other));
                }
            }
        }
        out.push(out_line);
    }

    let start = Location(
        0,
        find_single_path(out.first().context("Expected at least 1 row")?)
            .context("Failed to find start node")?,
    );
    let end = Location(
        out.len() - 1,
        find_single_path(out.last().context("Expected at least 1 row")?)
            .context("Failed to find end node")?,
    );

    Ok((Map(out), start, end))
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct NodeVisit {
    location: Location,
    prev: Option<Location>,
    distance: usize,
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let (grid, start, end) = parse_input(input).context("Failed to parse input")?;
    grid.longest_path::<Directed>(start, end, false)
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let (grid, start, end) = parse_input(input).context("Failed to parse input")?;
    grid.longest_path::<Undirected>(start, end, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(94));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(154));
        Ok(())
    }
}
