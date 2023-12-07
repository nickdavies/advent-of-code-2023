use anyhow::{anyhow, Context};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::str::FromStr;

advent_of_code::solution!(7);

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
enum Card {
    Jk,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
    J,
    Q,
    K,
    A,
}

impl TryFrom<char> for Card {
    type Error = anyhow::Error;

    fn try_from(other: char) -> Result<Self, Self::Error> {
        Ok(match other {
            'Z' => Self::Jk,
            '2' => Self::N2,
            '3' => Self::N3,
            '4' => Self::N4,
            '5' => Self::N5,
            '6' => Self::N6,
            '7' => Self::N7,
            '8' => Self::N8,
            '9' => Self::N9,
            'T' => Self::T,
            'J' => Self::J,
            'Q' => Self::Q,
            'K' => Self::K,
            'A' => Self::A,
            unknown => {
                return Err(anyhow!("Unknown card {}", unknown));
            }
        })
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfKind,
    FullHouse,
    FourOfKind,
    FiveOfKind,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Hand {
    cards: Vec<Card>,
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.optimal_hand_type().cmp(&other.optimal_hand_type()) {
            Ordering::Equal => self.cards.cmp(&other.cards),
            other => other,
        }
    }
}

impl Hand {
    #[allow(clippy::get_first)]
    fn optimal_hand_type(&self) -> HandType {
        let mut hist = BTreeMap::new();
        for card in &self.cards {
            hist.entry(card).and_modify(|e| *e += 1).or_insert(1_u32);
        }
        let jokers = hist.remove(&Card::Jk).unwrap_or(0);

        let mut hist: Vec<u32> = hist.into_values().collect();
        hist.sort_unstable_by_key(|item| std::cmp::Reverse(*item));

        match (
            jokers,
            hist.get(0).unwrap_or(&0),
            hist.get(1).unwrap_or(&0),
            hist.get(2).unwrap_or(&0),
            hist.get(3).unwrap_or(&0),
            hist.get(4).unwrap_or(&0),
        ) {
            (5, 0, ..) => HandType::FiveOfKind,
            // We can get five of a kind from any combination of 1 card
            // plus only jokers
            (0, 5, ..) => HandType::FiveOfKind,
            (1, 4, 0, ..) => HandType::FiveOfKind,
            (2, 3, 0, ..) => HandType::FiveOfKind,
            (3, 2, 0, ..) => HandType::FiveOfKind,
            (4, 1, 0, ..) => HandType::FiveOfKind,

            // We can make 4 of a kind by either having 0 jokers or
            (0, 4, 1, ..) => HandType::FourOfKind,
            (1, 3, 1, ..) => HandType::FourOfKind,
            (2, 2, 1, ..) => HandType::FourOfKind,
            (3, 1, 1, ..) => HandType::FourOfKind,

            // FullHouse can be made in quite a few different ways but there are
            // only two that are optimal. Either you have no jokers or you have
            // two pair + joker that you want to upgrade to 3 of a kind + pair.
            (0, 3, 2, ..) => HandType::FullHouse,
            (1, 2, 2, ..) => HandType::FullHouse,

            // Three of a kind
            (0, 3, 1, 1, ..) => HandType::ThreeOfKind,
            (1, 2, 1, 1, ..) => HandType::ThreeOfKind,
            (2, 1, 1, 1, ..) => HandType::ThreeOfKind,

            // Two Pair can't be made with jokers because
            // having even a single joker is enough to upgrade
            // one of the pairs to a 3 of a kind.
            (0, 2, 2, 1, ..) => HandType::TwoPair,

            // One Pair
            (0, 2, 1, 1, 1, 0) => HandType::OnePair,
            (1, 1, 1, 1, 1, 0) => HandType::OnePair,

            // High card by definition can't have any jokers
            (0, 1, 1, 1, 1, 1) => HandType::HighCard,

            _ => {
                panic!("Invalid histagram: {:?} with {} jokers", hist, jokers);
            }
        }
    }
}

impl FromStr for Hand {
    type Err = anyhow::Error;

    fn from_str(other: &str) -> Result<Self, Self::Err> {
        Ok(Hand {
            cards: other
                .chars()
                .map(Card::try_from)
                .collect::<Result<Vec<Card>, Self::Err>>()?,
        })
    }
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let mut data = input
        .lines()
        .map(|line| {
            let (hand, bet) = line.split_once(' ').context("Expected to find hand/bet")?;
            Ok((hand.parse()?, bet.parse()?))
        })
        .collect::<Result<Vec<(Hand, u32)>, anyhow::Error>>()
        .context("Failed to parse hand")?;

    let mut out = 0;
    data.sort();
    for (i, (_, bet)) in data.iter().enumerate() {
        out += (i as u32 + 1) * bet;
    }
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let mut data = input
        .lines()
        .map(|line| {
            let (hand, bet) = line.split_once(' ').context("Expected to find hand/bet")?;
            Ok((hand.replace('J', "Z").parse()?, bet.parse()?))
        })
        .collect::<Result<Vec<(Hand, u32)>, anyhow::Error>>()
        .context("Failed to parse hand/bet")?;

    let mut out = 0;
    data.sort();
    for (i, (hand, bet)) in data.iter().enumerate() {
        hand.optimal_hand_type();
        out += (i as u32 + 1) * bet;
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
        assert_eq!(result, Some(6440));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(5905));
        Ok(())
    }
}
