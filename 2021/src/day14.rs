#![allow(unused_imports)]

use std::collections::HashMap;

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take, take_while};
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (template, rules) = parsers::parse(p_instructions, input)?;

    fn pair_index(pair: [u8; 2]) -> usize {
        (pair[0] - b'A') as usize * 26 + (pair[1] - b'A') as usize
    }

    let mut rule_lookup = vec![0; 26 * 26];
    for rule in rules {
        rule_lookup[pair_index(rule.input)] = rule.output;
    }

    let mut pairs = vec![0; 26 * 26];
    for i in 0..template.len() - 1 {
        let input = [template[i], template[i + 1]];
        pairs[pair_index(input)] += 1;
    }

    let mut next = vec![0; 26 * 26];

    for _ in 0..10 {
        for p1 in b'A'..=b'Z' {
            for p2 in b'A'..=b'Z' {
                let pidx = pair_index([p1, p2]);
                let count = pairs[pidx];
                let output = rule_lookup[pidx];
                if output > 0 {
                    let pidx1 = pair_index([p1, output]);
                    let pidx2 = pair_index([output, p2]);
                    next[pidx1] += count;
                    next[pidx2] += count;
                } else {
                    next[pidx] += count;
                }
            }
        }
        std::mem::swap(&mut pairs, &mut next);
        next.iter_mut().for_each(|c| *c = 0);
    }

    let mut counts = vec![0; 26];
    // In the pair representation, every character is counted twice, except for
    // the first and last. Fortunately, the first and last character never
    // change, hence we can just easily count them extra here.
    counts[(template[0] - b'A') as usize] = 1;
    counts[(template[template.len() - 1] - b'A') as usize] = 1;

    for p1 in b'A'..=b'Z' {
        for p2 in b'A'..=b'Z' {
            let pidx = pair_index([p1, p2]);
            let count = pairs[pidx];
            counts[(p1 - b'A') as usize] += count;
            counts[(p2 - b'A') as usize] += count;
        }
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

    Ok(format!("{}", largest / 2 - smallest / 2))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (template, rules) = parsers::parse(p_instructions, input)?;

    // TODO: extract the parts common with part1

    fn pair_index(pair: [u8; 2]) -> usize {
        (pair[0] - b'A') as usize * 26 + (pair[1] - b'A') as usize
    }

    let mut rule_lookup = vec![0; 26 * 26];
    for rule in rules {
        rule_lookup[pair_index(rule.input)] = rule.output;
    }

    let mut pairs = vec![0_i64; 26 * 26];
    for i in 0..template.len() - 1 {
        let input = [template[i], template[i + 1]];
        pairs[pair_index(input)] += 1;
    }

    let mut next = vec![0_i64; 26 * 26];

    for _ in 0..40 {
        for p1 in b'A'..=b'Z' {
            for p2 in b'A'..=b'Z' {
                let pidx = pair_index([p1, p2]);
                let count = pairs[pidx];
                let output = rule_lookup[pidx];
                if output > 0 {
                    let pidx1 = pair_index([p1, output]);
                    let pidx2 = pair_index([output, p2]);
                    next[pidx1] += count;
                    next[pidx2] += count;
                } else {
                    next[pidx] += count;
                }
            }
        }
        std::mem::swap(&mut pairs, &mut next);
        next.iter_mut().for_each(|c| *c = 0);
    }

    let mut counts = vec![0; 26];
    counts[(template[0] - b'A') as usize] = 1;
    counts[(template[template.len() - 1] - b'A') as usize] = 1;

    for p1 in b'A'..=b'Z' {
        for p2 in b'A'..=b'Z' {
            let pidx = pair_index([p1, p2]);
            let count = pairs[pidx];
            counts[(p1 - b'A') as usize] += count;
            counts[(p2 - b'A') as usize] += count;
        }
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

    Ok(format!("{}", largest / 2 - smallest / 2))
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

crate::test_day!(crate::day14::RUN, "day14", "2851", "10002813279337");
