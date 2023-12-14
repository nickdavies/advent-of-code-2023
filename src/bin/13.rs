use anyhow::{anyhow, Result};

advent_of_code::solution!(13);

pub struct Map(Vec<Vec<char>>);

impl Map {
    fn get_columns(&self) -> Vec<Vec<char>> {
        let mut out = Vec::new();
        for _ in 0..self.0[0].len() {
            out.push(Vec::new());
        }
        for row in self.0.iter() {
            for (col_id, col) in row.iter().enumerate() {
                out[col_id].push(*col);
            }
        }

        out
    }
}

fn row_delta(left: &[char], right: &[char]) -> usize {
    let mut diffs = 0;
    for (l, r) in left.iter().zip(right) {
        if l != r {
            diffs += 1;
        }
    }
    diffs
}

fn find_reflection(data: &[Vec<char>], target_delta: usize) -> Option<usize> {
    for starting_left in 0..data.len() - 1 {
        let mut left = starting_left;
        let mut right = left + 1;
        let mut total_delta = row_delta(&data[left], &data[right]);
        while left > 0 && right < data.len() - 1 {
            left -= 1;
            right += 1;
            let row_delta = row_delta(&data[left], &data[right]);
            total_delta += row_delta;
            if total_delta > target_delta {
                break;
            }
        }
        if total_delta == target_delta {
            return Some(starting_left);
        }
    }
    None
}

pub fn parse_input(input: &str) -> Vec<Map> {
    let mut out = Vec::new();
    for chunk in input.split("\n\n") {
        let mut out_chunk = Vec::new();
        for line in chunk.lines() {
            out_chunk.push(line.chars().collect());
        }
        out.push(Map(out_chunk));
    }
    out
}

pub fn find_reflections(input: &str, target_delta: usize) -> Result<Option<usize>, anyhow::Error> {
    let mut out = 0;
    let maps = parse_input(input);

    for map in maps {
        if let Some(row) = find_reflection(&map.0, target_delta) {
            out += (row + 1) * 100;
        } else if let Some(col) = find_reflection(&map.get_columns(), target_delta) {
            out += col + 1;
        } else {
            return Err(anyhow!("Failed to find reflection on row or column"));
        }
    }
    Ok(Some(out))
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    find_reflections(input, 0)
}

pub fn part_two(input: &str) -> Result<Option<usize>, anyhow::Error> {
    find_reflections(input, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(405));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(400));
        Ok(())
    }
}
