#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let crab_counts = parse_crabs(input)?;

    let best_fuel = compute_fuel::<2>(&crab_counts);

    Ok(format!("{}", best_fuel))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let crab_counts = parse_crabs(input)?;

    let best_fuel = compute_fuel::<3>(&crab_counts);

    Ok(format!("{}", best_fuel))
}

fn compute_fuel<const ORDER: usize>(crab_counts: &[u32]) -> u64 {
    // Running total of fuel aggregation on the left
    let mut left = [0_u64; ORDER];

    // Running total of fuel aggregation on the right
    let mut right = crab_counts[1..]
        .iter()
        .rev()
        .fold([0_u64; ORDER], |mut right, crabs| {
            right[0] += *crabs as u64;
            for order_index in 1..ORDER {
                right[order_index] += right[order_index - 1];
            }
            right
        });

    let mut best_fuel = right[ORDER - 1];
    for (prev, cur) in crab_counts.iter().zip(crab_counts[1..].iter()) {
        // Take fuel from the right away
        for order_index in (1..ORDER).rev() {
            right[order_index] -= right[order_index - 1];
        }
        right[0] -= *cur as u64;
        // And put it to the left
        left[0] += *prev as u64;
        for order_index in 1..ORDER {
            left[order_index] += left[order_index - 1];
        }

        let fuel = left[ORDER - 1] + right[ORDER - 1];
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

crate::test_day!(crate::day7::RUN, "day7", "347509", "98257206");
