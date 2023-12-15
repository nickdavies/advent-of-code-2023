use anyhow::{Context, Result};

advent_of_code::solution!(15);

pub fn hash_segment(input: &str) -> Result<u8> {
    let mut hash: u32 = 0;
    for byte in input.bytes() {
        hash += byte as u32;
        hash *= 17;
        hash %= 256;
    }

    Ok(hash as u8)
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let mut out: u32 = 0;
    for segment in input.trim().split(',') {
        let hash = hash_segment(segment).context("Failed to hash segment")? as u32;
        out += hash;
    }
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let mut boxes: Vec<Vec<(&str, u8)>> = Vec::with_capacity(256);
    for _ in 0..256 {
        boxes.push(Vec::new());
    }

    for segment in input.trim().split(',') {
        if let Some(label) = segment.strip_suffix('-') {
            let hash = hash_segment(label).context("failed to hash segment")?;
            if let Some(idx) = boxes[hash as usize]
                .iter()
                .position(|(key, _)| key == &label)
            {
                boxes[hash as usize].remove(idx);
            }
        } else {
            let (label, number) = segment
                .split_once('=')
                .context("Expected to find = in segment")?;
            let focal_length: u8 = number
                .parse()
                .context("Expected value after = to be an u32")?;
            let hash = hash_segment(label).context("Failed to hash label")?;

            let b = &mut boxes[hash as usize];
            if let Some(idx) = b.iter().position(|(key, _)| key == &label) {
                b[idx].1 = focal_length;
            } else {
                b.push((label, focal_length));
            }
        }
    }

    let mut out = 0;
    for (box_num, lens_box) in boxes.iter().enumerate() {
        for (slot_num, (_, focal_len)) in lens_box.iter().enumerate() {
            out += (box_num + 1) * (slot_num + 1) * (*focal_len as usize);
        }
    }

    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() -> anyhow::Result<()> {
        let result = hash_segment("HASH")?;
        assert_eq!(result, 52);
        Ok(())
    }

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(1320));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(145));
        Ok(())
    }
}
