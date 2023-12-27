use advent_of_code::template::RunType;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{anyhow, Context, Result};

advent_of_code::solution!(20);

type PulseType = bool;

#[derive(Debug, Clone, Eq, PartialEq)]
enum NodeState {
    FlipFlop(FlipFlopState),
    Conjunction(ConjunctionState),
    Broadcaster,
}

impl NodeState {
    fn init_from_inputs(&mut self, inputs: &BTreeSet<String>) {
        if let NodeState::Conjunction(state) = self {
            for input in inputs {
                state.states.insert(input.clone(), false);
                state.num_low += 1;
            }
        }
    }

    fn send_pulse(
        &mut self,
        input: &str,
        outputs: &Vec<String>,
        pulse: PulseType,
    ) -> Result<Vec<(String, PulseType)>> {
        let mut out = Vec::new();
        match self {
            Self::FlipFlop(state) => {
                if !pulse {
                    state.value = !state.value;
                    for output in outputs {
                        out.push((output.clone(), state.value));
                    }
                }
            }
            Self::Conjunction(state) => {
                let single_state = state
                    .states
                    .get_mut(input)
                    .context("Didn't find specified input!")?;

                if pulse && !*single_state {
                    state.num_low -= 1;
                    state.num_high += 1;
                } else if !pulse && *single_state {
                    state.num_low += 1;
                    state.num_high -= 1;
                }
                *single_state = pulse;

                let send_pulse = !(state.num_low == 0 && state.num_high == state.states.len());

                for output in outputs {
                    out.push((output.clone(), send_pulse));
                }
            }
            Self::Broadcaster => {
                for output in outputs {
                    out.push((output.clone(), pulse));
                }
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct FlipFlopState {
    value: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct ConjunctionState {
    states: BTreeMap<String, bool>,
    num_low: usize,
    num_high: usize,
}

#[derive(Debug)]
struct Node {
    outputs: Vec<String>,
    inputs: BTreeSet<String>,
    node_state: NodeState,
}

impl Node {
    fn send_pulse(&mut self, input: &str, pulse: PulseType) -> Result<Vec<(String, PulseType)>> {
        self.node_state.send_pulse(input, &self.outputs, pulse)
    }
}

impl Node {
    fn from_outputs(outputs: Vec<String>, state: NodeState) -> Self {
        Self {
            outputs,
            inputs: BTreeSet::new(),
            node_state: state,
        }
    }

    fn set_inputs(&mut self, inputs: BTreeSet<String>) {
        self.inputs = inputs;
        self.node_state.init_from_inputs(&self.inputs);
    }
}

#[derive(Debug)]
struct Nodes(BTreeMap<String, Node>);

impl Nodes {
    fn send_pulses(&mut self, pulse: PulseType) -> Result<Vec<(String, String, PulseType)>> {
        let mut pulses = Vec::new();
        let mut to_process = VecDeque::new();
        to_process.push_back(("broadcaster".to_string(), "button".to_string(), pulse));

        while !to_process.is_empty() {
            let (target_node, input, pulse) = to_process.pop_front().unwrap();
            if let Some(node) = self.0.get_mut(&target_node) {
                let new_pulses = node
                    .send_pulse(&input, pulse)
                    .context("Failed to send pulse")?;

                for (new_target, new_pulse) in new_pulses {
                    to_process.push_back((new_target, target_node.clone(), new_pulse));
                }
            }
            pulses.push((target_node, input, pulse));
        }

        Ok(pulses)
    }

    fn find_output(&self) -> Option<(&String, &Node)> {
        for (name, node) in &self.0 {
            if node.outputs == ["rx"] {
                return Some((name, node));
            }
        }
        None
    }
}

fn parse_input(input: &str) -> Result<Nodes> {
    let mut all_inputs = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    for line in input.lines() {
        let (raw_name, data) = line
            .split_once(" -> ")
            .context("Expected -> in input line")?;
        let (name, state): (_, NodeState) = if let Some(name) = raw_name.strip_prefix('%') {
            (name, NodeState::FlipFlop(FlipFlopState::default()))
        } else if let Some(name) = raw_name.strip_prefix('&') {
            (name, NodeState::Conjunction(ConjunctionState::default()))
        } else if raw_name == "broadcaster" {
            (raw_name, NodeState::Broadcaster)
        } else {
            return Err(anyhow!("Got unexpected node named: {}", raw_name));
        };

        let outputs: Vec<String> = data.split(',').map(|s| s.trim().to_string()).collect();
        for output in &outputs {
            all_inputs
                .entry(output.clone())
                .or_insert_with(BTreeSet::new)
                .insert(name.to_string());
        }
        nodes.insert(name.to_string(), Node::from_outputs(outputs, state));
    }

    for (node_name, inputs) in &all_inputs {
        if let Some(node) = nodes.get_mut(node_name) {
            node.set_inputs(inputs.clone());
        }
    }

    Ok(Nodes(nodes))
}

// Copied from: https://github.com/TheAlgorithms/Rust/blob/master/src/math/lcm_of_n_numbers.rs
fn calculate_lcm(nums: &[u64]) -> u64 {
    if nums.len() == 1 {
        return nums[0];
    }
    let a = nums[0];
    let b = calculate_lcm(&nums[1..]);
    a * b / gcd_of_two_numbers(a, b)
}

// Copied from: https://github.com/TheAlgorithms/Rust/blob/master/src/math/lcm_of_n_numbers.rs
fn gcd_of_two_numbers(a: u64, b: u64) -> u64 {
    if b == 0 {
        return a;
    }
    gcd_of_two_numbers(b, a % b)
}

pub fn part_one(input: &str, _run_type: RunType) -> Result<Option<usize>, anyhow::Error> {
    let mut nodes = parse_input(input).context("Failed to parse input")?;
    let mut total_low_pulses = 0;
    let mut total_high_pulses = 0;
    for _ in 0..1000 {
        let new_pulses = nodes.send_pulses(false).context("Failed to send pulses")?;
        for (_, _, pulse) in new_pulses {
            if pulse {
                total_high_pulses += 1;
            } else {
                total_low_pulses += 1;
            }
        }
    }
    Ok(Some(total_low_pulses * total_high_pulses))
}

pub fn part_two(input: &str, _run_type: RunType) -> Result<Option<u64>, anyhow::Error> {
    let mut nodes = parse_input(input).context("Failed to parse input")?;
    let mut count = 0;

    let targets = nodes
        .find_output()
        .context("Expected to find output node")?
        .1
        .inputs
        .clone();

    let mut to_process = VecDeque::new();
    let mut cycles = BTreeMap::new();
    loop {
        if to_process.is_empty() {
            to_process.push_back(("broadcaster".to_string(), "button".to_string(), false));
            count += 1;
        }
        let (target_node, input, pulse) = to_process.pop_front().unwrap();
        if targets.contains(&input) && pulse && !cycles.contains_key(&input) {
            cycles.insert(input.to_string(), count);
            if cycles.len() == targets.len() {
                break;
            }
        }
        if let Some(node) = nodes.0.get_mut(&target_node) {
            let new_pulses = node
                .send_pulse(&input, pulse)
                .context("Failed to send pulse")?;

            for (new_target, new_pulse) in new_pulses {
                to_process.push_back((new_target, target_node.clone(), new_pulse));
            }
        }
    }
    let nums: Vec<u64> = cycles.values().copied().collect();
    Ok(Some(calculate_lcm(&nums)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one_example_1() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(32000000));
        Ok(())
    }

    #[test]
    fn test_part_one_example_2() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 3);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(11687500));
        Ok(())
    }

    #[test]
    fn test_part_one_real_input() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 4);
        let result = part_one(input, RunType::Example)?;
        assert_eq!(result, Some(899848294));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input, RunType::Example)?;
        assert_eq!(result, Some(244055946148853));
        Ok(())
    }
}
