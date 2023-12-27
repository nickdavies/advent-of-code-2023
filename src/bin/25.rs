use anyhow::{Context, Result};
use petgraph::graphmap::UnGraphMap;
use std::collections::BTreeSet;

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

fn external_connections(
    graph: &UnGraphMap<&str, ()>,
    node_set: &BTreeSet<&str>,
    target: &str,
) -> usize {
    let mut external = 0;
    for (_, edge, _) in graph.edges_directed(target, petgraph::Direction::Outgoing) {
        if !node_set.contains(edge) {
            external += 1;
        }
    }
    external
}

pub fn part_one(input: &str) -> Result<Option<usize>, anyhow::Error> {
    let graph = parse_input(input).context("Failed to parse input")?;

    let mut lhs: BTreeSet<&str> = graph.nodes().collect();
    let mut external_nodes = 0;

    while external_nodes != 3 {
        let (_, highest_external) = lhs
            .iter()
            .map(|n| (external_connections(&graph, &lhs, n), n))
            .max()
            .unwrap();
        lhs.remove(*highest_external);

        external_nodes = lhs
            .iter()
            .map(|n| external_connections(&graph, &lhs, n))
            .sum();
    }

    Ok(Some(lhs.len() * (graph.node_count() - lhs.len())))
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
}
