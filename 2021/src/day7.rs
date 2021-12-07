#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let crab_counts = parse_crabs(input)?;

    let best_fuel = compute_target(&crab_counts, abs_diff);

    Ok(best_fuel as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let crab_counts = parse_crabs(input)?;

    let best_fuel = compute_target(&crab_counts, tri_sum_diff);

    Ok(best_fuel as i64)
}

fn tri_sum_diff(target: usize, pos: usize) -> usize {
    let delta = abs_diff(target, pos);
    (delta * (delta + 1)) / 2
}

fn abs_diff(target: usize, pos: usize) -> usize {
    if pos > target {
        pos - target
    } else {
        target - pos
    }
}

fn compute_target<F: Fn(usize, usize) -> usize>(crab_counts: &[u32], distance: F) -> u64 {
    // brute force
    let mut best_fuel = u64::MAX;
    for target in 0..crab_counts.len() {
        let mut fuel = 0_u64;
        for (pos, count) in crab_counts.iter().enumerate() {
            fuel += *count as u64 * distance(pos, target) as u64;
        }
        if fuel < best_fuel {
            best_fuel = fuel;
        }
    }
    best_fuel
}

fn parse_crabs(input: &[u8]) -> Result<Vec<u32>, anyhow::Error> {
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
    Ok(crab_counts)
}

crate::test_day!(crate::day7::RUN, "day7", 347509, 0);
