use anyhow::Context;

advent_of_code::solution!(1);

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let mut out = 0;
    for line in input.lines() {
        let mut first_digit = None;
        let mut last_digit = None;

        for c in line.chars() {
            match (c.to_digit(10), first_digit, last_digit) {
                (None, _, _) => continue,
                (Some(digit), None, _) => {
                    first_digit = Some(digit);
                    last_digit = Some(digit);
                }
                (Some(digit), Some(_), _) => {
                    last_digit = Some(digit);
                }
            }
        }

        out += first_digit.context("Expected to find first digit")? * 10;
        out += last_digit.context("Expected to find second digit")?;
    }
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let mut out = 0;
    for line in input.lines() {
        let mut first_digit = None;
        let mut last_digit = None;
        for (i, c) in line.char_indices() {
            let digit = match c.to_digit(10) {
                Some(digit) => Some(digit),
                None => {
                    let substr = &line[i..];
                    if substr.starts_with("one") {
                        Some(1)
                    } else if substr.starts_with("two") {
                        Some(2)
                    } else if substr.starts_with("three") {
                        Some(3)
                    } else if substr.starts_with("four") {
                        Some(4)
                    } else if substr.starts_with("five") {
                        Some(5)
                    } else if substr.starts_with("six") {
                        Some(6)
                    } else if substr.starts_with("seven") {
                        Some(7)
                    } else if substr.starts_with("eight") {
                        Some(8)
                    } else if substr.starts_with("nine") {
                        Some(9)
                    } else {
                        None
                    }
                }
            };
            match (digit, first_digit, last_digit) {
                (None, _, _) => continue,
                (Some(digit), None, _) => {
                    first_digit = Some(digit);
                    last_digit = Some(digit);
                }
                (Some(digit), Some(_), _) => {
                    last_digit = Some(digit);
                }
            }
        }

        out += first_digit.unwrap() * 10;
        out += last_digit.unwrap();
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
        assert_eq!(result, Some(142));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(281));
        Ok(())
    }
}
