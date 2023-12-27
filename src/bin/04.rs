use advent_of_code::template::RunType;
use anyhow::{anyhow, Context};
use std::collections::BTreeSet;

advent_of_code::solution!(4);

#[derive(Clone)]
pub struct GameData {
    card_id: u32,
    winning_numbers: BTreeSet<u32>,
    my_numbers: BTreeSet<u32>,
}

impl GameData {
    fn matches(&self) -> usize {
        self.my_numbers.intersection(&self.winning_numbers).count()
    }
}

impl std::str::FromStr for GameData {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (left, right) = input.split_once(": ").context("All lines must have a :")?;
        let card_id: u32 = left
            .rsplit_once(' ')
            .context("Failed to extract card id")?
            .1
            .parse()
            .context("Failed to convert card to int")?;

        let (winning, my) = right.split_once(" | ").context("Expected to find split")?;

        let mut winning_numbers = BTreeSet::new();
        for num in winning.split_ascii_whitespace() {
            let num: u32 = num
                .trim()
                .parse()
                .context("failed to parse winning numbers")?;
            if winning_numbers.contains(&num) {
                return Err(anyhow!("Duplicate winning key {}", num));
            } else {
                winning_numbers.insert(num);
            }
        }
        let mut my_numbers = BTreeSet::new();
        for num in my.split_ascii_whitespace() {
            let num: u32 = num.trim().parse().context("failed to parse my number")?;
            if my_numbers.contains(&num) {
                return Err(anyhow!("Duplicate my key {}", num));
            } else {
                my_numbers.insert(num);
            }
        }
        Ok(GameData {
            card_id,
            winning_numbers,
            my_numbers,
        })
    }
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<u32>, anyhow::Error> {
    let mut out = 0;
    for line in input.lines() {
        let game_data: GameData = line.parse().unwrap();

        let mut value = 0;
        for _ in game_data
            .my_numbers
            .intersection(&game_data.winning_numbers)
        {
            if value == 0 {
                value += 1;
            } else {
                value *= 2;
            }
        }
        out += value;
    }
    Ok(Some(out))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u32>, anyhow::Error> {
    let mut cards = Vec::new();
    let mut card_counts = Vec::new();
    card_counts.push(0); // Fake card 0
    for line in input.lines() {
        let game_data: GameData = line.parse().unwrap();
        cards.push(game_data);
        card_counts.push(1);
    }
    let mut out = 0;
    for card in &cards {
        let added_cards = card_counts[(card.card_id) as usize];
        out += added_cards;
        for i in 0..card.matches() {
            let new_card = card.card_id + i as u32 + 1;
            if new_card <= cards.len() as u32 {
                card_counts[new_card as usize] += added_cards;
            }
        }
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(13));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(30));
        Ok(())
    }
}
