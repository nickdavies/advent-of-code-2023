use anyhow::{anyhow, Context};
use std::collections::BTreeMap;

advent_of_code::solution!(2);

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum Color {
    Red,
    Blue,
    Green,
}

impl std::str::FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "blue" => Ok(Self::Blue),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            other => Err(anyhow!("Unknown color {}", other)),
        }
    }
}

#[derive(Debug)]
struct GameData {
    id: u32,
    combos: Vec<BTreeMap<Color, u32>>,
}

impl std::str::FromStr for GameData {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (left, right) = input.split_once(": ").context("All lines must have a :")?;
        let game_id: u32 = left
            .split_once(' ')
            .context("Failed to extract game id")?
            .1
            .parse()
            .context("Failed to convert game_id to int")?;

        let mut combos = Vec::new();
        for combination in right.split(';') {
            let mut cube_counts = BTreeMap::new();
            for cube in combination.split(',') {
                let (count, color) = cube
                    .trim()
                    .split_once(' ')
                    .context("Cube pair was missing a space")?;

                let color: Color = color.parse().context("Failed to parse cube color")?;
                if cube_counts.contains_key(&color) {
                    return Err(anyhow!("Duplicate color {:?} found", color));
                }
                cube_counts.insert(color, count.parse().context("Failed to parse cube count")?);
            }
            combos.push(cube_counts);
        }
        Ok(GameData {
            id: game_id,
            combos,
        })
    }
}

impl GameData {
    fn is_possible(&self, limits: fn(&Color) -> u32) -> bool {
        for combo in &self.combos {
            let impossible = combo.iter().any(|(color, count)| *count > limits(color));

            if impossible {
                return false;
            }
        }
        true
    }

    fn minimum_cubes(&self, target_color: Color) -> Option<u32> {
        self.combos
            .iter()
            .filter_map(|cube_counts| cube_counts.get(&target_color))
            .max()
            .copied()
    }

    fn game_power(&self) -> u32 {
        self.minimum_cubes(Color::Red).unwrap_or(0)
            * self.minimum_cubes(Color::Blue).unwrap_or(0)
            * self.minimum_cubes(Color::Green).unwrap_or(0)
    }
}

fn parse_games(input: &str) -> anyhow::Result<Vec<GameData>> {
    let mut out = Vec::new();
    for line in input.lines() {
        out.push(line.parse().context("Failed to parse game data")?);
    }
    Ok(out)
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let data = parse_games(input).context("failed to parse input data")?;

    let mut possible = 0;
    for game in data {
        let is_possible = game.is_possible(|color| match color {
            Color::Red => 12,
            Color::Green => 13,
            Color::Blue => 14,
        });
        if is_possible {
            possible += game.id;
        }
    }
    Ok(Some(possible))
}

pub fn part_two(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let data = parse_games(input).context("failed to parse input data")?;
    let total: u32 = data.iter().map(|game| game.game_power()).sum();

    Ok(Some(total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(8));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(2286));
        Ok(())
    }
}
