use anyhow::{anyhow, Context, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{
    alpha1, char as nom_char, line_ending, multispace0, one_of, u32 as nom_u32,
};
use nom::combinator::all_consuming;
use nom::error::context as nom_context;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::{Finish, IResult};
use std::collections::BTreeMap;

advent_of_code::solution!(19);

type ParseResult<I, O> = IResult<I, O, nom::error::VerboseError<I>>;
type Rules<'a> = BTreeMap<&'a str, Workflow<'a>>;

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
        value: u32,
        outcome: Outcome<'a>,
    },
    GreaterThan {
        key: Key,
        value: u32,
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
struct Part {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}

impl Part {
    fn get_key(&self, key: &Key) -> u32 {
        match key {
            Key::X => self.x,
            Key::M => self.m,
            Key::A => self.a,
            Key::S => self.s,
        }
    }

    fn sum(&self) -> u32 {
        self.x + self.m + self.a + self.s
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
            let (input, value) = nom_u32(input)?;
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

fn nom_rules(input: &str) -> ParseResult<&str, Rules<'_>> {
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

    Ok((input, all_rules.into_iter().collect()))
}

fn nom_part_value(input: &str) -> ParseResult<&str, (char, u32)> {
    let (input, key) = one_of("xmas")(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, value) = nom_u32(input)?;

    Ok((input, (key, value)))
}

fn nom_parts(input: &str) -> ParseResult<&str, Vec<Part>> {
    let parser = move |input| {
        let (input, _) = nom_context("parsing start of part {", tag("{"))(input)?;
        let (input, kv) =
            separated_list1(tag(","), nom_context("parsing single part", nom_part_value))(input)?;
        let (input, _) = nom_context("parsing end of part }", tag("}"))(input)?;

        let kv: BTreeMap<char, u32> = kv.into_iter().collect();

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

fn parse_input(input: &str) -> anyhow::Result<(Rules, Vec<Part>)> {
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

pub fn part_one(input: &str) -> Result<Option<u32>, anyhow::Error> {
    let (rules, parts) = parse_input(input).context("failed to parse input")?;
    println!("{:#?}", rules);
    println!("{:?}", parts);
    let mut out = 0;
    for part in parts {
        let mut workflow_name = "in";
        let accepted = loop {
            let workflow = rules
                .get(workflow_name)
                .context(format!("Got invalid workflow {}", workflow_name))?;
            match workflow
                .try_match(&part)
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
    Ok(Some(out))
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
        assert_eq!(result, Some(19114));
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
