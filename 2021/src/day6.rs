#![allow(unused)]

use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::terminated;
use nom::IResult;

use crate::{parsers, Day};
use std::cmp::Ordering;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    parse_and_sim(input, 80)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    parse_and_sim(input, 256)
}

fn parse_and_sim(input: &[u8], num_days: u32) -> Result<i64, anyhow::Error> {
    let fish_ages = parsers::parse(
        terminated(
            separated_list0(tag(","), parsers::u32),
            take_while(|x: u8| x.is_ascii_whitespace()),
        ),
        input,
    )?;
    Ok(simulate(&fish_ages, num_days) as i64)
}

fn simulate(fish_ages: &[u32], num_days: u32) -> u64 {
    let mut pop = Population::new();
    for fish in fish_ages {
        pop.fish_by_timer[*fish as usize] += 1;
    }
    for _ in 0..num_days {
        pop = pop.advance();
    }
    pop.fish_by_timer.iter().sum()
}

const MAX_AGE: usize = 8;

struct Population {
    fish_by_timer: [u64; MAX_AGE + 1],
}

impl Population {
    pub fn new() -> Self {
        Self {
            fish_by_timer: [0; MAX_AGE + 1],
        }
    }

    pub fn advance(self) -> Self {
        // This is multiplying the count vector with the growth (transition)
        // matrix.
        Population {
            fish_by_timer: [
                self.fish_by_timer[1],
                self.fish_by_timer[2],
                self.fish_by_timer[3],
                self.fish_by_timer[4],
                self.fish_by_timer[5],
                self.fish_by_timer[6],
                self.fish_by_timer[7] + self.fish_by_timer[0],
                self.fish_by_timer[8],
                self.fish_by_timer[0],
            ],
        }
    }
}

crate::test_day!(crate::day6::RUN, "day6", 395627, 1767323539209);
