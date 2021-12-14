#![allow(unused_imports)]

use std::collections::HashMap;

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take, take_while};
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let (template, rules) = parsers::parse(p_instructions, input)?;

    fn pair_index(pair: [u8; 2]) -> usize {
        (pair[0] - b'A') as usize * 26 + (pair[1] - b'A') as usize
    }

    let mut rule_lookup = rules
        .iter()
        .map(|r| (r.input, r.output))
        .collect::<HashMap<_, _>>();

    let mut poly = template.to_owned();
    let mut next = Vec::with_capacity(2 * poly.len());

    for _ in 0..10 {
        for i in 0..poly.len() - 1 {
            let pair = [poly[i], poly[i + 1]];
            next.push(poly[i]);
            if let Some(output) = rule_lookup.get(&pair) {
                next.push(*output);
            }
        }
        next.push(poly[poly.len() - 1]);
        std::mem::swap(&mut poly, &mut next);
        next.clear();
    }

    let mut counts = vec![0; 26];

    for b in poly.iter() {
        counts[(*b - b'A') as usize] += 1;
    }

    let (smallest, largest) = counts.iter().copied().fold((0, 0), |(s, l), c| {
        if c == 0 {
            (s, l)
        } else {
            (
                if s == 0 || c < s { c } else { s },
                if l == 0 || c > l { c } else { l },
            )
        }
    });

    Ok((largest - smallest) as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let (template, rules) = parsers::parse(p_instructions, input)?;

    fn pair_index(pair: [u8; 2]) -> usize {
        (pair[0] - b'A') as usize * 26 + (pair[1] - b'A') as usize
    }

    let mut pair_counts = todo!();
}

fn p_instructions(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<Rule>)> {
    separated_pair(p_template, parsers::newline, p_rules)(input)
}

fn p_template(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(take_while(|b| matches!(b, b'A'..=b'Z')), parsers::newline)(input)
}

fn p_rule(input: &[u8]) -> IResult<&[u8], Rule> {
    // TODO: apparently nom has no way of matching a single byte with a predicate :-(
    map_opt(
        separated_pair(take(2usize), tag(" -> "), take(1usize)),
        |(input, output): (&[u8], &[u8])| {
            if matches!(input[0], b'A'..=b'Z')
                && matches!(input[1], b'A'..=b'Z')
                && matches!(output[0], b'A'..=b'Z')
            {
                Some(Rule {
                    input: [input[0], input[1]],
                    output: output[0],
                })
            } else {
                None
            }
        },
    )(input)
}

fn p_rules(input: &[u8]) -> IResult<&[u8], Vec<Rule>> {
    many0(terminated(p_rule, parsers::newline))(input)
}

struct Rule {
    input: [u8; 2],
    output: u8,
}

crate::test_day!(crate::day14::RUN, "day14", 2851, 0);
