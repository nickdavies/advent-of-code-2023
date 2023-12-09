use nom::character::complete::{i32 as nom_i32, line_ending, space1};
use nom::combinator::all_consuming;
use nom::multi::{many1, separated_list1};
use nom::{Finish, IResult};

advent_of_code::solution!(9);

#[derive(Debug, Clone)]
struct Sequence(Vec<i32>);

type FnGetNext = fn(&Sequence, i32) -> i32;

impl Sequence {
    fn step(&self) -> (Sequence, bool) {
        let mut out = Vec::new();
        let mut all_zero = true;
        for window in self.0.windows(2) {
            let diff = window[1] - window[0];
            if diff != 0 {
                all_zero = false;
            }
            out.push(diff);
        }

        (Sequence(out), all_zero)
    }

    fn extrapolate(self, get_next: FnGetNext) -> i32 {
        let mut layers = vec![self];
        loop {
            let (next_layer, all_zero) = layers.last().unwrap().step();
            layers.push(next_layer);
            if all_zero {
                break;
            }
        }

        let mut next_value = 0;
        for layer in layers.iter().rev() {
            next_value = get_next(layer, next_value);
        }

        next_value
    }
}

fn nom_line(input: &str) -> IResult<&str, Sequence> {
    let (input, result) = separated_list1(space1, nom_i32)(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, Sequence(result)))
}

fn parse_input(input: &str) -> anyhow::Result<Vec<Sequence>> {
    match all_consuming(many1(nom_line))(input).finish() {
        Ok(output) => Ok(output.1),
        Err(e) => Err(nom::error::Error::new(e.input.to_string(), e.code).into()),
    }
}

fn solve(input: &str, get_next: FnGetNext) -> anyhow::Result<Option<i32>> {
    let data = parse_input(input)?;

    let mut out = 0;
    for row in data.into_iter() {
        out += row.extrapolate(get_next);
    }
    Ok(Some(out))
}

pub fn part_one(input: &str) -> Result<Option<i32>, anyhow::Error> {
    solve(input, |seq, next| seq.0.last().unwrap() + next)
}

pub fn part_two(input: &str) -> Result<Option<i32>, anyhow::Error> {
    solve(input, |seq, next| seq.0.first().unwrap() - next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(114));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(2));
        Ok(())
    }
}
