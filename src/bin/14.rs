use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;

advent_of_code::solution!(14);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Value {
    Rolling,
    Fixed,
}

type InnerMap = Vec<Vec<Option<Value>>>;

#[derive(Debug, PartialEq)]
struct Map(InnerMap);

impl Map {
    fn roll_rows<'a, T, C>(input: T, other_len: usize) -> InnerMap
    where
        T: Iterator<Item = C>,
        C: Iterator<Item = &'a Option<Value>>,
    {
        let mut out: InnerMap = Vec::new();
        let mut next_idxs = vec![0; other_len];

        for (row_id, row) in input.enumerate() {
            out.push(Vec::new());
            for (col_id, col) in row.enumerate() {
                match col {
                    Some(Value::Rolling) => {
                        out[row_id].push(None);
                        out[next_idxs[col_id]][col_id] = Some(Value::Rolling);
                        next_idxs[col_id] += 1;
                    }
                    Some(Value::Fixed) => {
                        out[row_id].push(Some(Value::Fixed));
                        next_idxs[col_id] = row_id + 1;
                    }
                    None => {
                        out[row_id].push(None);
                    }
                }
            }
        }

        out
    }

    fn roll_north(&self) -> Self {
        Self(Self::roll_rows(
            self.0.iter().map(|r| r.iter()),
            self.0[0].len(),
        ))
    }

    fn roll_south(&self) -> Self {
        let mut inner = Self::roll_rows(self.0.iter().rev().map(|r| r.iter()), self.0[0].len());
        inner.reverse();
        Self(inner)
    }

    fn roll_west(&self) -> Self {
        let col_iters =
            (0..self.0[0].len()).map(|col_id| self.0.iter().map(move |row| &row[col_id]));
        let inner = Self::roll_rows(col_iters, self.0.len());
        let inner: InnerMap = (0..inner.len())
            .map(|col_id| inner.iter().map(move |row| row[col_id].clone()).collect())
            .collect();

        Self(inner)
    }

    fn roll_east(&self) -> Self {
        let col_iters = (0..self.0[0].len())
            .rev()
            .map(|col_id| self.0.iter().map(move |row| &row[col_id]));
        let inner = Self::roll_rows(col_iters, self.0.len());
        let inner: InnerMap = (0..inner.len())
            .map(|col_id| {
                inner
                    .iter()
                    .rev()
                    .map(move |row| row[col_id].clone())
                    .collect()
            })
            .collect();

        Self(inner)
    }

    fn run_cycle(&self) -> Self {
        self.roll_north().roll_west().roll_south().roll_east()
    }

    fn calculate_north_weight(&self) -> usize {
        let mut weight = 0;
        let max = self.0.len();
        for (row_num, row) in self.0.iter().enumerate() {
            for col in row.iter().flatten() {
                if let Value::Rolling = col {
                    weight += max - row_num;
                }
            }
        }
        weight
    }

    #[allow(dead_code)]
    fn print(&self) {
        for row in &self.0 {
            for col in row {
                let char = match col {
                    Some(Value::Fixed) => '#',
                    Some(Value::Rolling) => 'O',
                    None => '.',
                };
                print!("{}", char);
            }
            println!();
        }
    }
}

fn parse_input(input: &str) -> Result<Map> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_row = Vec::new();
        for char in line.chars() {
            out_row.push(match char {
                '.' => None,
                '#' => Some(Value::Fixed),
                'O' => Some(Value::Rolling),
                other => {
                    return Err(anyhow!("Invalid input char: {} found", other));
                }
            });
        }
        out.push(out_row);
    }
    Ok(Map(out))
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    Ok(Some(
        parse_input(input)
            .context("Failed to parse input")?
            .roll_north()
            .calculate_north_weight(),
    ))
}

pub fn part_two(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let mut map = parse_input(input).context("Failed to parse input")?;
    let mut seen: HashMap<InnerMap, usize> = HashMap::new();
    let mut cycle_idx = None;

    for idx in 0..1000000000 {
        if let Some(seen_idx) = seen.get(&map.0) {
            cycle_idx = Some((*seen_idx, idx));
            break;
        } else {
            seen.insert(map.0.clone(), idx);
        }
        map = map.run_cycle();
    }

    let current_idx = match cycle_idx {
        Some(cycle_idx) => {
            let total_remaining = 1000000000 - 1 - cycle_idx.1;
            let off_by = total_remaining % (cycle_idx.1 - cycle_idx.0);

            1000000000 - 1 - off_by
        }
        None => 1000000000 - 1,
    };
    println!("Cycle of {:?} gets us to {}", cycle_idx, current_idx);
    for _ in current_idx..1000000000 {
        map = map.run_cycle();
    }

    Ok(Some(map.calculate_north_weight()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(136));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(64));
        Ok(())
    }

    #[test]
    fn test_single_rotation() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let expected = &advent_of_code::template::read_file_part("examples", DAY, 3);

        let map = parse_input(input).context("Failed to parse input")?;
        let north_map = map.roll_north();
        let west_map = north_map.roll_west();
        let south_map = west_map.roll_south();
        let east_map = south_map.roll_east();

        let expected_map = parse_input(expected).context("Failed to parse expected")?;
        assert_eq!(east_map, expected_map);
        Ok(())
    }
}
