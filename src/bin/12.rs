use nom::branch::alt;
use nom::character::complete::{char as nom_char, line_ending, multispace0, u32 as nom_u32};
use nom::combinator::all_consuming;
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::Parser;
use nom::{Finish, IResult};
use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

advent_of_code::solution!(12);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Pattern {
    Unknown,
    Broken,
    Operational,
}

impl TryFrom<&char> for Pattern {
    type Error = anyhow::Error;

    fn try_from(other: &char) -> Result<Self, Self::Error> {
        match other {
            '?' => Ok(Pattern::Unknown),
            '#' => Ok(Pattern::Broken),
            '.' => Ok(Pattern::Operational),
            other => Err(anyhow!("Got unexpected pattern character '{}'", other)),
        }
    }
}

fn parse_pattern(input: &str) -> IResult<&str, Vec<Pattern>> {
    let (input, entries) = many1(alt((nom_char('?'), nom_char('.'), nom_char('#'))))(input)?;

    let mut out = Vec::new();
    for entry in &entries {
        out.push(entry.try_into().unwrap());
    }

    Ok((input, out))
}

fn parse_line(input: &str) -> IResult<&str, (Vec<Pattern>, Vec<u32>)> {
    separated_pair(
        parse_pattern,
        nom_char(' '),
        separated_list1(nom_char(','), nom_u32),
    )(input)
}

fn parse_input(input: &str) -> Result<Vec<(Vec<Pattern>, Vec<u32>)>> {
    let result = all_consuming(separated_list1(line_ending, parse_line).and(multispace0))(input);
    match result.finish() {
        Ok(output) => Ok(output.1 .0),
        Err(e) => Err(nom::error::Error::new(e.input.to_string(), e.code).into()),
    }
}

fn consume_n_broken(mut remaining_pattern: &[Pattern], target: u32) -> Option<&[Pattern]> {
    for _ in 0..target {
        remaining_pattern = match remaining_pattern.split_first() {
            None => {
                return None;
            }
            Some((first, remaining)) => match first {
                Pattern::Operational => {
                    return None;
                }
                Pattern::Broken | Pattern::Unknown => remaining,
            },
        };
    }
    match remaining_pattern.split_first() {
        // If we ended at the end of the pattern that is ok
        None => Some(remaining_pattern),
        // If we end at an unknown that is also ok because it must be Operational.
        // we can continue checking from the pattern after onwards
        Some((Pattern::Unknown, remaining_pattern)) => Some(remaining_pattern),
        // We must match a block exactly for this to count
        Some((Pattern::Broken, _)) => None,
        Some((Pattern::Operational, _)) => Some(remaining_pattern),
    }
}

fn get_combos<'a, 'b>(
    remaining_pattern: &'a [Pattern],
    remaining_nums: &'b [u32],
    cache: &mut HashMap<(&'a [Pattern], &'b [u32]), usize>,
) -> usize {
    let key = (remaining_pattern, remaining_nums);
    if let Some(cached) = cache.get(&key) {
        return *cached;
    }
    if remaining_pattern.is_empty() {
        if remaining_nums.is_empty() {
            return 1;
        } else {
            return 0;
        }
    }

    let (first_pat, rest_pat) = remaining_pattern.split_at(1);
    let first_pat = &first_pat[0];

    let out = match first_pat {
        Pattern::Operational => get_combos(rest_pat, remaining_nums, cache),
        // We must match an exact number form remaining nums
        Pattern::Broken => {
            let (first_num, rest_nums) = match remaining_nums.split_first() {
                Some(res) => res,
                None => {
                    return 0;
                }
            };

            // consume first_num.
            match consume_n_broken(remaining_pattern, *first_num) {
                Some(remaining_pattern) => get_combos(remaining_pattern, rest_nums, cache),
                None => 0,
            }
        }
        Pattern::Unknown => {
            // Either we treat this as a working spring and just continue processing the
            // rest
            let options_if_operational = get_combos(rest_pat, remaining_nums, cache);

            // Or we treat this as the first broken one and consume next n
            let options_if_broken = match remaining_nums.split_first() {
                Some((first_num, rest_nums)) => {
                    let res = consume_n_broken(remaining_pattern, *first_num);
                    match res {
                        Some(remaining_pattern) => get_combos(remaining_pattern, rest_nums, cache),
                        None => 0,
                    }
                }
                None => 0,
            };
            options_if_operational + options_if_broken
        }
    };
    cache.insert(key, out);
    out
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let data = parse_input(input).context("Failed to parse input")?;
    let mut out = 0;

    let mut cache = HashMap::new();
    for (pattern, nums) in data.iter() {
        let combos = get_combos(pattern, nums, &mut cache);
        out += combos;
    }
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let data = parse_input(input).context("Failed to parse input")?;

    let mut out = 0;
    for (pattern, nums) in data {
        let mut cache = HashMap::new();
        let mut new_pattern = Vec::new();
        let mut new_nums = Vec::new();
        for i in 0..5 {
            new_pattern.extend(pattern.clone());
            if i != 4 {
                new_pattern.push(Pattern::Unknown);
            }
            new_nums.extend(nums.clone());
        }
        let combos = get_combos(&new_pattern, &new_nums, &mut cache);
        out += combos;
    }

    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(21));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(525152));
        Ok(())
    }
}
