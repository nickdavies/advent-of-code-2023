use advent_of_code::template::RunType;
use anyhow::Context;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::num::ParseIntError;

advent_of_code::solution!(5);

#[derive(Debug, Clone)]
pub struct SparseMap(Vec<(u32, u32, u32)>);

impl SparseMap {
    fn lookup(&self, key: u32) -> Option<u32> {
        for (source_start, dest_start, length) in &self.0 {
            let delta = key as i64 - *source_start as i64;
            if delta < 0 {
                return None;
            } else if delta >= *length as i64 {
                continue;
            }

            return Some(dest_start + delta as u32);
        }
        None
    }

    fn rev_lookup(&self, key: u32) -> Option<u32> {
        for (source_start, dest_start, length) in &self.0 {
            if key >= *dest_start && key <= dest_start + length {
                let delta = key - dest_start;
                return Some(source_start + delta);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct InputData {
    seed_to_soil: SparseMap,
    soil_to_fert: SparseMap,
    fert_to_water: SparseMap,
    water_to_light: SparseMap,
    light_to_temp: SparseMap,
    temp_to_humid: SparseMap,
    humid_to_location: SparseMap,
}

impl InputData {
    fn location_for_seed(&self, seed: u32) -> u32 {
        let soil = self.seed_to_soil.lookup(seed).unwrap_or(seed);
        let fert = self.soil_to_fert.lookup(soil).unwrap_or(soil);
        let water = self.fert_to_water.lookup(fert).unwrap_or(fert);
        let light = self.water_to_light.lookup(water).unwrap_or(water);
        let temp = self.light_to_temp.lookup(light).unwrap_or(light);
        let humid = self.temp_to_humid.lookup(temp).unwrap_or(temp);

        self.humid_to_location.lookup(humid).unwrap_or(humid)
    }

    fn seed_for_location(&self, location: u32) -> u32 {
        let humid = self
            .humid_to_location
            .rev_lookup(location)
            .unwrap_or(location);
        let temp = self.temp_to_humid.rev_lookup(humid).unwrap_or(humid);
        let light = self.light_to_temp.rev_lookup(temp).unwrap_or(temp);
        let water = self.water_to_light.rev_lookup(light).unwrap_or(light);
        let fert = self.fert_to_water.rev_lookup(water).unwrap_or(water);
        let soil = self.soil_to_fert.rev_lookup(fert).unwrap_or(fert);
        self.seed_to_soil.rev_lookup(soil).unwrap_or(soil)
    }
}

pub fn parse_map_section(section: &str) -> anyhow::Result<SparseMap> {
    let data = section
        .split_once("map:\n")
        .context("Expected to find map marker")?
        .1;

    let mut out = Vec::new();
    for line in data.lines() {
        let (dest_range_start, remainder) = line
            .split_once(' ')
            .context("Expected at least 2 numbers in map line")?;
        let (source_range_start, length) = remainder
            .split_once(' ')
            .context("Expected at least 3 numbers in map line")?;

        let dest_range_start: u32 = dest_range_start.parse()?;
        let source_range_start: u32 = source_range_start.parse()?;

        out.push((source_range_start, dest_range_start, length.parse()?));
    }
    out.sort();

    Ok(SparseMap(out))
}

pub fn parse_maps(mut sections: std::str::Split<'_, &str>) -> Result<InputData, anyhow::Error> {
    Ok(InputData {
        seed_to_soil: parse_map_section(sections.next().context("expected map section")?)?,
        soil_to_fert: parse_map_section(sections.next().context("expected map section")?)?,
        fert_to_water: parse_map_section(sections.next().context("expected map section")?)?,
        water_to_light: parse_map_section(sections.next().context("expected map section")?)?,
        light_to_temp: parse_map_section(sections.next().context("expected map section")?)?,
        temp_to_humid: parse_map_section(sections.next().context("expected map section")?)?,
        humid_to_location: parse_map_section(sections.next().context("expected map section")?)?,
    })
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<u32>, anyhow::Error> {
    let mut sections = input.split("\n\n");

    let seeds: BTreeSet<u32> = sections
        .next()
        .context("Expected seeds section")?
        .split_once(": ")
        .context("Expected : to divide name")?
        .1
        .split_whitespace()
        .map(|s| s.parse())
        .collect::<Result<BTreeSet<u32>, ParseIntError>>()?;

    let data = parse_maps(sections)?;

    let min = seeds
        .iter()
        .map(|seed| data.location_for_seed(*seed))
        .min()
        .context("Expected a minimum location")?;

    Ok(Some(min))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u32>, anyhow::Error> {
    let mut sections = input.split("\n\n");

    let mut seeds = sections
        .next()
        .context("Expected seeds section")?
        .split_once(": ")
        .context("Expected : to divide name")?
        .1
        .split_whitespace()
        .chunks(2)
        .into_iter()
        .map(|mut chunk| {
            let start = chunk.next().unwrap().parse()?;
            let len = chunk.next().unwrap().parse()?;
            Ok((start, start, len))
        })
        .collect::<Result<Vec<(u32, u32, u32)>, ParseIntError>>()?;

    seeds.sort();

    let seeds = SparseMap(seeds);

    let data = parse_maps(sections)?;

    let first_seed = std::ops::Range {
        start: 0,
        end: u32::MAX,
    }
    .map(|location| data.seed_for_location(location))
    .filter_map(|seed| seeds.lookup(seed))
    .next()
    .context("Expected at least one location")?;

    Ok(Some(data.location_for_seed(first_seed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(35));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(46));
        Ok(())
    }
}
