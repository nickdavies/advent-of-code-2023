use advent_of_code::template::RunType;
use anyhow::anyhow;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, anychar, line_ending, multispace0};
use nom::combinator::map_res;
use nom::error::ParseError;
use nom::multi::{many1, many_till};
use nom::{Finish, IResult};
use std::collections::BTreeMap;

advent_of_code::solution!(8);

type Map = BTreeMap<String, (String, String)>;

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

fn nom_map_line(input: &str) -> IResult<&str, (String, (String, String))> {
    let (input, key) = alphanumeric1(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, left_key) = alphanumeric1(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, right_key) = alphanumeric1(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, _) = line_ending(input)?;

    Ok((
        input,
        (
            key.to_string(),
            (left_key.to_string(), right_key.to_string()),
        ),
    ))
}

fn nom_input(input: &str) -> IResult<&str, (Vec<Direction>, Map), nom::error::Error<&str>> {
    let single_direction = map_res(anychar, |c| match c {
        'L' => Ok(Direction::Left),
        'R' => Ok(Direction::Right),
        _ => Err(nom::error::Error::from_error_kind(
            input,
            nom::error::ErrorKind::Fail,
        )),
    });

    let (input, (directions, _)) = many_till(single_direction, line_ending)(input)?;
    let (input, _) = multispace0(input)?;

    let (input, mapping) = many1(nom_map_line)(input)?;

    IResult::Ok((input, (directions, mapping.into_iter().collect())))
}

fn parse_input(input: &str) -> anyhow::Result<(Vec<Direction>, Map)> {
    match nom_input(input).finish() {
        Ok(output) => Ok(output.1),
        Err(e) => Err(nom::error::Error::new(e.input.to_string(), e.code).into()),
    }
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<u32>, anyhow::Error> {
    let (directions, mapping) = parse_input(input)?;

    let mut out = 0;
    let mut current = "AAA";
    for direction in directions.iter().cycle() {
        out += 1;
        current = match mapping.get(current) {
            Some((left, right)) => match direction {
                Direction::Left => left,
                Direction::Right => right,
            },
            None => {
                return Err(anyhow!("Found non-existant key: {}", current));
            }
        };

        if current == "ZZZ" {
            return Ok(Some(out));
        }
    }
    Ok(None)
}

fn find_cycle<'a>(
    directions: &[Direction],
    mut start: &'a str,
    mapping: &'a Map,
) -> anyhow::Result<(usize, usize)> {
    let mut seen = BTreeMap::new();
    let mut out = Vec::new();
    for (idx, direction) in directions.iter().enumerate().cycle() {
        if let Some(cycle_start_idx) = seen.get(&(idx, start)) {
            return Ok((*cycle_start_idx, seen.len()));
        }
        seen.insert((idx, start), seen.len());
        if start.ends_with('Z') {
            out.push((idx, start));
        }

        start = match mapping.get(start) {
            Some((left, right)) => match direction {
                Direction::Left => left,
                Direction::Right => right,
            },
            None => {
                return Err(anyhow!("Found non-existant key: {}", start));
            }
        };
    }

    unreachable!();
}

// Copied from: https://github.com/TheAlgorithms/Rust/blob/master/src/math/lcm_of_n_numbers.rs
pub fn calculate_lcm(nums: &[u64]) -> u64 {
    if nums.len() == 1 {
        return nums[0];
    }
    let a = nums[0];
    let b = calculate_lcm(&nums[1..]);
    a * b / gcd_of_two_numbers(a, b)
}

// Copied from: https://github.com/TheAlgorithms/Rust/blob/master/src/math/lcm_of_n_numbers.rs
fn gcd_of_two_numbers(a: u64, b: u64) -> u64 {
    if b == 0 {
        return a;
    }
    gcd_of_two_numbers(b, a % b)
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let (directions, mapping) = parse_input(input)?;

    let mut all_current = Vec::new();
    for key in mapping.keys() {
        if key.ends_with('A') {
            all_current.push(key);
        }
    }

    let mut cycle_lengths = Vec::new();
    for start in all_current.iter() {
        let (cycle_start_idx, total_path_length) = find_cycle(&directions, start, &mapping)?;
        cycle_lengths.push(total_path_length as u64 - cycle_start_idx as u64);
    }

    let lcm = calculate_lcm(&cycle_lengths);
    Ok(Some(lcm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(2));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(6));
        Ok(())
    }
}
