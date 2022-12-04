#![allow(unused)]

use std::ops::RangeInclusive;

use anyhow::bail;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::fold_many0,
    sequence::{separated_pair, terminated},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let fully_overlapping = parsers::parse(
        parse_ranges_count_if(|r1, r2| fully_contains(r1, r2) || fully_contains(r2, r1)),
        input,
    )?;

    Ok(fully_overlapping.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let overlapping = parsers::parse(parse_ranges_count_if(|r1, r2| !disjoint(r1, r2)), input)?;

    Ok(overlapping.to_string())
}

fn parse_ranges_count_if(
    cond: impl Fn(&RangeInclusive<u32>, &RangeInclusive<u32>) -> bool,
) -> impl Fn(&[u8]) -> IResult<&[u8], u32> {
    move |input| {
        fold_many0(
            terminated(separated_pair(parse_range, tag(","), parse_range), newline),
            || 0,
            |count, (r1, r2)| {
                if cond(&r1, &r2) {
                    count + 1
                } else {
                    count
                }
            },
        )(input)
    }
}

fn parse_range(input: &[u8]) -> IResult<&[u8], RangeInclusive<u32>> {
    map(
        separated_pair(parsers::u32, tag("-"), parsers::u32),
        |(from, to)| from..=to,
    )(input)
}

fn fully_contains(outer: &RangeInclusive<u32>, inner: &RangeInclusive<u32>) -> bool {
    outer.start() <= inner.start() && outer.end() >= inner.end()
}

fn disjoint(r1: &RangeInclusive<u32>, r2: &RangeInclusive<u32>) -> bool {
    r1.start() > r2.end() || r1.end() < r2.start()
}

crate::test_day!(RUN, "day4", "507", "897");
