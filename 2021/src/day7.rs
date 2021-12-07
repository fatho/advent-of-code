#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let crab_positions = parsers::parse(
        terminated(
            separated_list0(tag(","), parsers::u32),
            take_while(|x: u8| x.is_ascii_whitespace()),
        ),
        input,
    )?;

    let num_pos = crab_positions.iter().max().map_or(0, |x| x + 1);
    let mut crab_counts = vec![0; num_pos as usize];

    for pos in crab_positions {
        crab_counts[pos as usize] += 1;
    }

    // brute force
    let mut best_fuel = u64::MAX;
    for target in 0..crab_counts.len() {
        let mut fuel = 0_u64;
        for (pos, count) in crab_counts.iter().enumerate() {
            let delta = if pos > target {
                pos - target
            } else {
                target - pos
            };
            fuel += *count as u64 * delta as u64;
        }
        if fuel < best_fuel {
            best_fuel = fuel;
        }
    }

    Ok(best_fuel as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(|_| Ok((b"", 0)), input)
}

crate::test_day!(crate::day7::RUN, "day7", 347509, 0);
