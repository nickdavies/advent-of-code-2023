use advent_of_code::template::RunType;
use anyhow::Context;

advent_of_code::solution!(6);

pub fn extract_lines(input: &str) -> anyhow::Result<(&str, &str)> {
    let mut lines = input.lines();
    let times = lines
        .next()
        .context("expected time line")?
        .split_once(':')
        .context("expected to find : in first line")?
        .1
        .trim();

    let distances = lines
        .next()
        .context("expected distance line")?
        .split_once(':')
        .context("expected to find : in second line")?
        .1
        .trim();

    Ok((times, distances))
}

fn calculate_race_options(time: u64, distance: u64) -> u64 {
    let inner = (((time * time) - 4 * distance) as f64).sqrt();
    let mut min_time = ((time as f64 - inner) / 2.0).ceil() as u64;
    let mut max_time = ((time as f64 + inner) / 2.0).floor() as u64;
    if (time - min_time) * min_time == distance {
        min_time += 1;
    }
    if (time - max_time) * max_time == distance {
        max_time -= 1;
    }
    max_time - min_time + 1
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let (times, distances) = extract_lines(input)?;

    let times = times.split_whitespace().map(|s| s.parse::<u64>());
    let distances = distances.split_whitespace().map(|s| s.parse::<u64>());

    let mut out = 1;
    for (time, distance) in times.zip(distances) {
        out *= calculate_race_options(time?, distance?);
    }
    Ok(Some(out))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let (time, distance) = extract_lines(input)?;
    let time: u64 = time.replace(' ', "").parse()?;
    let distance: u64 = distance.replace(' ', "").parse()?;
    Ok(Some(calculate_race_options(time, distance)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(288));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(71503));
        Ok(())
    }
}
