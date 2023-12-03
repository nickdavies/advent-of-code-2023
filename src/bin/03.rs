use anyhow::anyhow;
use std::collections::BTreeMap;

advent_of_code::solution!(3);

pub fn extract_numbers(line: &str) -> Vec<(usize, usize, u32)> {
    let mut numbers = Vec::new();

    let mut start = None;
    let mut number = 0;
    for (char_num, c) in line.char_indices() {
        match (c.to_digit(10), start) {
            (Some(digit), Some(_)) => {
                number *= 10;
                number += digit;
            }
            (Some(digit), None) => {
                start = Some(char_num);
                number = digit;
            }
            (None, Some(start_char)) => {
                numbers.push((start_char, char_num - 1, number));
                start = None;
            }
            (None, None) => continue,
        }
    }
    if let Some(start_char) = start {
        numbers.push((start_char, line.len() - 1, number));
    }
    numbers
}

fn build_symbols(input: &str, is_symbol: fn(char) -> bool) -> Vec<Vec<bool>> {
    let mut symbols = Vec::new();
    for line in input.lines() {
        let mut line_symbols = Vec::with_capacity(line.len());
        for c in line.chars() {
            line_symbols.push(is_symbol(c));
        }
        symbols.push(line_symbols);
    }

    symbols
}

fn test_surroundings<F>(input: &str, mut test: F)
where
    F: FnMut(u32, &Vec<usize>, &Vec<usize>),
{
    let lines: Vec<&str> = input.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        let mut rows = Vec::new();
        if line_num != 0 {
            rows.push(line_num - 1);
        }
        rows.push(line_num);
        if line_num + 1 < lines.len() {
            rows.push(line_num + 1);
        }

        for (start_num, end_num, number) in extract_numbers(line) {
            let mut cols = Vec::new();
            if start_num != 0 {
                cols.push(start_num - 1);
            }
            for col in start_num..=end_num {
                cols.push(col);
            }
            if end_num + 1 < line.len() {
                cols.push(end_num + 1);
            }

            test(number, &rows, &cols);
        }
    }
}

fn any_matching(rows: &Vec<usize>, cols: &Vec<usize>, symbols: &[Vec<bool>]) -> bool {
    for row in rows {
        for col in cols {
            if symbols[*row][*col] {
                return true;
            }
        }
    }
    false
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let symbols = build_symbols(input, |c| !(c.is_ascii_digit() || c == '.'));

    let mut out = 0;
    test_surroundings(input, |number, rows, cols| {
        if any_matching(rows, cols, &symbols) {
            out += number;
        }
    });
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let symbols = build_symbols(input, |c| c == '*');

    let mut gears = BTreeMap::new();

    test_surroundings(input, |number, rows, cols| {
        for row in rows {
            for col in cols {
                if symbols[*row][*col] {
                    gears
                        .entry((*row, *col))
                        .and_modify(|e: &mut Vec<u32>| e.push(number))
                        .or_insert_with(|| vec![number]);
                }
            }
        }
    });

    let mut out = 0;
    for (_, members) in gears.into_iter() {
        match members.len() {
            1 => continue,
            2 => {
                out += members.iter().product::<u32>();
            }
            other => {
                return Err(anyhow!(
                    "Unexpected number of members in gear ratio: {}",
                    other
                ));
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
        let result = part_one(input)?;
        assert_eq!(result, Some(4361));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(467835));
        Ok(())
    }
}
