use anyhow::{anyhow, Context, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{
    alpha1, char as nom_char, line_ending, multispace0, one_of, u64 as nom_u64,
};
use nom::combinator::all_consuming;
use nom::error::context as nom_context;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::{Finish, IResult};
use std::collections::BTreeMap;

advent_of_code::solution!(19);

type ParseResult<I, O> = IResult<I, O, nom::error::VerboseError<I>>;

#[derive(Debug, Clone, Copy)]
enum Key {
    X,
    M,
    A,
    S,
}

impl TryFrom<char> for Key {
    type Error = anyhow::Error;

    fn try_from(other: char) -> Result<Self, Self::Error> {
        match other {
            'x' => Ok(Key::X),
            'm' => Ok(Key::M),
            'a' => Ok(Key::A),
            's' => Ok(Key::S),
            other => Err(anyhow!("Got unexpected key {}", other)),
        }
    }
}

#[derive(Debug)]
enum Outcome<'a> {
    Accept,
    Reject,
    Redirect(&'a str),
}

#[derive(Debug)]
enum Rule<'a> {
    Outcome(Outcome<'a>),
    LessThan {
        key: Key,
        value: u64,
        outcome: Outcome<'a>,
    },
    GreaterThan {
        key: Key,
        value: u64,
        outcome: Outcome<'a>,
    },
}

impl<'a> Rule<'a> {
    fn try_match(&'a self, part: &Part) -> Option<&'a Outcome> {
        match self {
            Rule::Outcome(o) => Some(o),
            Rule::LessThan {
                key,
                value,
                outcome,
            } => {
                if &part.get_key(key) < value {
                    Some(outcome)
                } else {
                    None
                }
            }
            Rule::GreaterThan {
                key,
                value,
                outcome,
            } => {
                if &part.get_key(key) > value {
                    Some(outcome)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug)]
struct Workflow<'a>(Vec<Rule<'a>>);

impl<'a> Workflow<'a> {
    fn try_match(&'a self, part: &Part) -> Result<&'a Outcome<'a>> {
        for rule in &self.0 {
            if let Some(outcome) = rule.try_match(part) {
                return Ok(outcome);
            }
        }
        Err(anyhow!("Got to end of workflow without any matches"))
    }
}

#[derive(Debug)]
struct Workflows<'a>(BTreeMap<&'a str, Workflow<'a>>);

impl<'a> Workflows<'a> {
    fn sum_matching_parts(&self, parts: &[Part]) -> Result<u64> {
        let mut out = 0;
        for part in parts {
            let mut workflow_name = "in";
            let accepted = loop {
                let workflow = self
                    .0
                    .get(workflow_name)
                    .context(format!("Got invalid workflow {}", workflow_name))?;
                match workflow
                    .try_match(part)
                    .context(format!("failed to match workflow {}", workflow_name))?
                {
                    Outcome::Accept => {
                        break true;
                    }
                    Outcome::Reject => {
                        break false;
                    }
                    Outcome::Redirect(target) => {
                        workflow_name = target;
                    }
                }
            };

            if accepted {
                out += part.sum();
            }
        }
        Ok(out)
    }

    fn get_matching_ranges(&self, workflow: &Workflow, mut current: PartRange) -> u64 {
        let mut total = 0;
        for rule in &workflow.0 {
            let (matching, nonmatching) = current.split_on(rule);
            if let Some((outcome, matching_range)) = matching {
                match outcome {
                    Outcome::Accept => {
                        total += matching_range.options();
                    }
                    Outcome::Reject => {}
                    Outcome::Redirect(target) => {
                        let workflow = self.0.get(target).unwrap();
                        total += self.get_matching_ranges(workflow, matching_range);
                    }
                }
            }

            if let Some(nonmatching_range) = nonmatching {
                current = nonmatching_range;
            } else {
                break;
            }
        }
        total
    }
}

#[derive(Debug)]
struct Part {
    x: u64,
    m: u64,
    a: u64,
    s: u64,
}

impl Part {
    fn get_key(&self, key: &Key) -> u64 {
        match key {
            Key::X => self.x,
            Key::M => self.m,
            Key::A => self.a,
            Key::S => self.s,
        }
    }

    fn sum(&self) -> u64 {
        self.x + self.m + self.a + self.s
    }
}

#[derive(Debug, Clone)]
struct Range(u64, u64);

impl Range {
    fn new_if_valid(lower: u64, upper: u64) -> Option<Self> {
        if lower >= upper {
            None
        } else {
            Some(Self(lower, upper))
        }
    }

    fn split_lt(&self, at: u64) -> (Option<Self>, Option<Self>) {
        (
            Self::new_if_valid(self.0, at - 1),
            Self::new_if_valid(at, self.1),
        )
    }

    fn split_gt(&self, at: u64) -> (Option<Self>, Option<Self>) {
        (
            Self::new_if_valid(self.0, at),
            Self::new_if_valid(at + 1, self.1),
        )
    }

    fn options(&self) -> u64 {
        if self.0 == 0 {
            self.1 - self.0
        } else {
            self.1 - self.0 + 1
        }
    }
}

#[derive(Debug, Clone)]
struct PartRange {
    x: Range,
    m: Range,
    a: Range,
    s: Range,
}

impl PartRange {
    fn options(&self) -> u64 {
        self.x.options() * self.m.options() * self.a.options() * self.s.options()
    }

    fn get_key(&self, key: &Key) -> &Range {
        match key {
            Key::X => &self.x,
            Key::M => &self.m,
            Key::A => &self.a,
            Key::S => &self.s,
        }
    }

    fn set_key(&mut self, key: &Key, value: Range) {
        match key {
            Key::X => self.x = value,
            Key::M => self.m = value,
            Key::A => self.a = value,
            Key::S => self.s = value,
        }
    }

    fn split_on<'a>(
        &'a self,
        rule: &'a Rule,
    ) -> (Option<(&Outcome, PartRange)>, Option<PartRange>) {
        let (key, outcome, matching, nonmatching) = match rule {
            Rule::Outcome(o) => {
                return (Some((o, self.clone())), None);
            }
            Rule::LessThan {
                key,
                value,
                outcome,
            } => {
                let (matching, nonmatching) = self.get_key(key).split_lt(*value);
                (key, outcome, matching, nonmatching)
            }
            Rule::GreaterThan {
                key,
                value,
                outcome,
            } => {
                let (nonmatching, matching) = self.get_key(key).split_gt(*value);
                (key, outcome, matching, nonmatching)
            }
        };
        let matching = if let Some(range) = matching {
            let mut new_range = self.clone();
            new_range.set_key(key, range);
            Some((outcome, new_range))
        } else {
            None
        };

        let nonmatching = if let Some(range) = nonmatching {
            let mut new_range = self.clone();
            new_range.set_key(key, range);
            Some(new_range)
        } else {
            None
        };

        (matching, nonmatching)
    }
}

fn nom_outcome<'a>(input: &'a str) -> ParseResult<&str, Outcome<'a>> {
    alt((
        move |input: &'a str| {
            let (input, _) = nom_char('A')(input)?;
            Ok((input, Outcome::Accept))
        },
        move |input: &'a str| {
            let (input, _) = nom_char('R')(input)?;
            Ok((input, Outcome::Reject))
        },
        move |input: &'a str| {
            let (input, name) = alpha1(input)?;
            Ok((input, Outcome::Redirect(name)))
        },
    ))(input)
}

fn nom_rule<'a>(input: &'a str) -> ParseResult<&str, Rule<'a>> {
    alt((
        move |input: &'a str| {
            let (input, key) = one_of("xmas")(input)?;
            let (input, op) = one_of("<>")(input)?;
            let (input, value) = nom_u64(input)?;
            let (input, _) = tag(":")(input)?;
            let (input, outcome) = nom_outcome(input)?;

            Ok((
                input,
                match op {
                    '<' => Rule::LessThan {
                        key: key.try_into().unwrap(),
                        value,
                        outcome,
                    },
                    '>' => Rule::GreaterThan {
                        key: key.try_into().unwrap(),
                        value,
                        outcome,
                    },
                    other => {
                        panic!("Got unexpected char {} after matching only <>", other);
                    }
                },
            ))
        },
        move |input| {
            let (input, outcome) = nom_outcome(input)?;
            Ok((input, Rule::Outcome(outcome)))
        },
    ))(input)
}

fn nom_rules(input: &str) -> ParseResult<&str, Workflows<'_>> {
    let parser = move |input| {
        let (input, name) = nom_context("parsing rule name", alpha1)(input)?;
        let (input, _) = nom_context("parsing start of rules list {", tag("{"))(input)?;
        let (input, rules) =
            separated_list1(tag(","), nom_context("while parsing single rule", nom_rule))(input)?;
        let (input, _) = nom_context("while parsing ending } of rules list", tag("}"))(input)?;

        Ok((input, (name, Workflow(rules))))
    };
    let (input, all_rules) =
        separated_list1(line_ending, nom_context("parsing rules line", parser))(input)?;

    Ok((input, Workflows(all_rules.into_iter().collect())))
}

fn nom_part_value(input: &str) -> ParseResult<&str, (char, u64)> {
    let (input, key) = one_of("xmas")(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, value) = nom_u64(input)?;

    Ok((input, (key, value)))
}

fn nom_parts(input: &str) -> ParseResult<&str, Vec<Part>> {
    let parser = move |input| {
        let (input, _) = nom_context("parsing start of part {", tag("{"))(input)?;
        let (input, kv) =
            separated_list1(tag(","), nom_context("parsing single part", nom_part_value))(input)?;
        let (input, _) = nom_context("parsing end of part }", tag("}"))(input)?;

        let kv: BTreeMap<char, u64> = kv.into_iter().collect();

        Ok((
            input,
            Part {
                x: *kv.get(&'x').unwrap(),
                m: *kv.get(&'m').unwrap(),
                a: *kv.get(&'a').unwrap(),
                s: *kv.get(&'s').unwrap(),
            },
        ))
    };
    separated_list1(line_ending, nom_context("parsing single part line", parser))(input)
}

fn parse_input(input: &str) -> anyhow::Result<(Workflows<'_>, Vec<Part>)> {
    let parser = move |input| {
        let (input, data) = separated_pair(
            nom_context("parsing rules", nom_rules),
            tag("\n\n"),
            nom_context("parsing parts", nom_parts),
        )(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, data))
    };

    match all_consuming(parser)(input).finish() {
        Ok(output) => Ok(output.1),
        Err(e) => Err(anyhow!(nom::error::convert_error(input, e))),
    }
}

pub fn part_one(input: &str) -> Result<Option<u64>, anyhow::Error> {
    let (workflows, parts) = parse_input(input).context("failed to parse input")?;
    let out = workflows
        .sum_matching_parts(&parts)
        .context("Failed to calculate matching parts")?;
    Ok(Some(out))
}

pub fn part_two(input: &str) -> Result<Option<u64>, anyhow::Error> {
    let (workflows, _) = parse_input(input).context("failed to parse input")?;

    let full_range = PartRange {
        x: Range(0, 4000),
        m: Range(0, 4000),
        a: Range(0, 4000),
        s: Range(0, 4000),
    };

    let workflow = workflows.0.get("in").unwrap();
    let total = workflows.get_matching_ranges(workflow, full_range);
    Ok(Some(total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 1);
        let result = part_one(input)?;
        assert_eq!(result, Some(19114));
        Ok(())
    }

    #[test]
    fn test_part_two() -> anyhow::Result<()> {
        let input = &advent_of_code::template::read_file_part("examples", DAY, 2);
        let result = part_two(input)?;
        assert_eq!(result, Some(167409079868000));
        Ok(())
    }
}
