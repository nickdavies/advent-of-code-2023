use anyhow::{Context, Result};
use petgraph::graphmap::UnGraphMap;

advent_of_code::solution!(25);

fn parse_input(input: &str) -> Result<UnGraphMap<&str, ()>> {
    let mut out = UnGraphMap::new();
    for line in input.lines() {
        let (lhs, other) = line
            .split_once(": ")
            .context("Expected to find : separator")?;

        let lhs = lhs.trim();
        out.add_node(lhs);
        for rhs in other.split_whitespace() {
            let rhs = rhs.trim();
            out.add_node(rhs);
            out.add_edge(lhs, rhs, ());
        }
    }

    Ok(out)
}

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let graph = parse_input(input).context("Failed to parse input")?;
    println!("{:?}", graph);
    std::fs::write(
        "graph",
        format!("{:?}", petgraph::dot::Dot::with_config(&graph, &[])),
    )
    .context("Failed to write graph to file")?;
    Ok(None)
}

pub fn part_two(_input: &str) -> Result<Option<u32>, anyhow::Error> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(54));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, None);
        Ok(())
    }
}
