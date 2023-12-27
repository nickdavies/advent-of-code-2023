use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
advent_of_code::solution!(11);

type Map = Vec<Vec<bool>>;

fn parse_input(input: &str) -> Result<Map> {
    let mut out = Vec::new();
    for line in input.lines() {
        let mut out_row = Vec::new();
        for char in line.chars() {
            out_row.push(match char {
                '.' => false,
                '#' => true,
                other => {
                    return Err(anyhow!("Got unexpected character {}", other));
                }
            });
        }
        out.push(out_row);
    }

    Ok(out)
}

pub fn find_galaxies(map: Map, expansion_ratio: usize) -> BTreeSet<(usize, usize)> {
    let mut empty_rows = BTreeSet::new();
    let mut col_counts = BTreeMap::new();
    for (row_id, row) in map.iter().enumerate() {
        let mut empty_row = true;
        for (col_id, col) in row.iter().enumerate() {
            let entry = col_counts.entry(col_id).or_insert(0);
            if *col {
                *entry += 1;
                empty_row = false;
            }
        }
        if empty_row {
            empty_rows.insert(row_id);
        }
    }

    let empty_cols: BTreeSet<usize> = col_counts
        .into_iter()
        .filter_map(|(col, count)| if count == 0 { Some(col) } else { None })
        .collect();

    let mut galaxies = BTreeSet::new();
    let mut actual_row = 0;
    for (row_num, row) in map.iter().enumerate() {
        let mut actual_col = 0;
        for (col_num, col) in row.iter().enumerate() {
            if *col {
                galaxies.insert((actual_row, actual_col));
            }
            if empty_cols.contains(&col_num) {
                actual_col += expansion_ratio;
            } else {
                actual_col += 1;
            }
        }
        if empty_rows.contains(&row_num) {
            actual_row += expansion_ratio;
        } else {
            actual_row += 1;
        }
    }

    galaxies
}

pub fn find_distances(input: &str, expansion_ratio: usize) -> Result<Option<usize>, anyhow::Error> {
    let map = parse_input(input).context("Failed to parse input")?;
    let galaxies = find_galaxies(map, expansion_ratio);
    let mut out = 0;
    for galaxy_1 in galaxies.iter() {
        for galaxy_2 in galaxies.iter() {
            if galaxy_1 <= galaxy_2 {
                continue;
            }
            let delta_row = galaxy_2.0.abs_diff(galaxy_1.0);
            let delta_col = galaxy_2.1.abs_diff(galaxy_1.1);
            let dist = delta_row + delta_col;
            out += dist;
        }
    }

    Ok(Some(out))
}
pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    find_distances(input, 2)
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    find_distances(input, 1000000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(374));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(82000210));
        Ok(())
    }
}
