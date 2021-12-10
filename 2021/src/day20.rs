#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(|_| Ok((b"", 0)), input)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(|_| Ok((b"", 0)), input)
}

crate::test_day!(crate::day20::RUN, "day20", 0, 0);