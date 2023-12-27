use advent_of_code::template::RunType;
use anyhow::{anyhow, Context, Result};

advent_of_code::solution!(18);

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}
#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
struct Location(i64, i64);

impl Location {
    fn go_direction(&self, direction: &Direction, distance: usize) -> Location {
        match direction {
            Direction::North => Location(self.0 - distance as i64, self.1),
            Direction::East => Location(self.0, self.1 + distance as i64),
            Direction::South => Location(self.0 + distance as i64, self.1),
            Direction::West => Location(self.0, self.1 - distance as i64),
        }
    }
}

struct DigInstruction {
    direction: Direction,
    distance: usize,
}

impl DigInstruction {
    fn from_normal(direction: &str, distance: &str) -> Result<Self> {
        Ok(Self {
            direction: match direction {
                "U" => Direction::North,
                "R" => Direction::East,
                "D" => Direction::South,
                "L" => Direction::West,
                other => {
                    return Err(anyhow!("Got unexpected value {} for direction", other));
                }
            },
            distance: distance.parse().context("Expected distance to be an int")?,
        })
    }

    fn from_colour_code(code: &str) -> Result<Self> {
        let code = code
            .strip_prefix('(')
            .context("Expected colour to start with (")?
            .strip_prefix('#')
            .context("Expected colour to start with (#")?
            .strip_suffix(')')
            .context("Expected colour to end with )")?;

        let (distance, direction) = code.split_at(5);
        Ok(DigInstruction {
            direction: match direction {
                "3" => Direction::North,
                "0" => Direction::East,
                "1" => Direction::South,
                "2" => Direction::West,
                other => {
                    return Err(anyhow!("Got unexpected value {} for direction", other));
                }
            },
            distance: usize::from_str_radix(distance, 16)
                .context(format!("failed to parse colour {} code as hex", distance))?,
        })
    }
}

fn parse_input(
    input: &str,
    builder: fn(std::str::SplitWhitespace<'_>) -> Result<DigInstruction>,
) -> Result<Vec<DigInstruction>> {
    let mut out = Vec::new();
    for line in input.lines() {
        out.push(builder(line.split_whitespace())?);
    }
    Ok(out)
}

#[derive(Debug)]
struct Path {
    locations: Vec<Location>,
    total_points: u64,
}

impl Path {
    fn from_instructions(instructions: &[DigInstruction]) -> Path {
        let mut out = Path {
            locations: Vec::new(),
            total_points: 1,
        };

        out.locations.push(Location(0, 0));

        for instruction in instructions {
            out.total_points += instruction.distance as u64;
            let location = out
                .locations
                .last()
                .unwrap()
                .go_direction(&instruction.direction, instruction.distance);
            out.locations.push(location);
        }

        out
    }

    fn get_area(&self) -> u64 {
        let mut sum = 0;
        for window in self.locations.windows(2) {
            let l1 = &window[0];
            let l2 = &window[1];
            let det = (l1.0 * l2.1) - (l1.1 * l2.0);
            sum += det;
        }

        let abs_sum = sum.abs_diff(0) + self.total_points;

        if abs_sum % 2 == 0 {
            abs_sum / 2
        } else {
            (abs_sum + 1) / 2
        }
    }
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let instructions = parse_input(input, |mut segments| {
        DigInstruction::from_normal(
            segments.next().context("Expected a direction")?,
            segments.next().context("Expected a distance")?,
        )
    })
    .context("failed to parse input")?;
    let path = Path::from_instructions(&instructions);
    Ok(Some(path.get_area()))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let instructions = parse_input(input, |mut segments| {
        segments.next().context("Expected a direction")?;
        segments.next().context("Expected a distance")?;
        DigInstruction::from_colour_code(segments.next().context("Expected colour code")?)
    })
    .context("failed to parse input")?;
    let path = Path::from_instructions(&instructions);
    Ok(Some(path.get_area()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(62));
        Ok(())
    }

    #[test]
    fn test_part_one_custom() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 3);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(78));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(952408144115));
        Ok(())
    }
}
