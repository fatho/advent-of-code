#![allow(unused)]

use anyhow::bail;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::eof,
    multi::fold_many1, sequence::terminated,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let max = parsers::parse(
        fold_many1(
            terminated(parse_inventory, alt((tag("\n"), eof))),
            || 0,
            |prev_max, cur| prev_max.max(cur),
        ),
        input,
    )?;
    Ok(max.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let top3 = parsers::parse(
        fold_many1(
            terminated(parse_inventory, alt((tag("\n"), eof))),
            || [0; 3],
            |mut top3, cur| {
                let (min_pos, _) = top3.iter().enumerate().min_by_key(|(i, val)| *val).unwrap();
                top3[min_pos] = top3[min_pos].max(cur);
                top3
            },
        ),
        input,
    )?;
    Ok(top3.iter().sum::<u64>().to_string())
}

fn parse_inventory(input: &[u8]) -> nom::IResult<&[u8], u64> {
    fold_many1(terminated(parsers::u64, char('\n')), || 0, |acc, i| acc + i)(input)
}

#[test]
fn test_parse_inventory() {
    assert_eq!(
        parse_inventory(b"10\n20\n30\n"),
        Ok((b"".as_slice(), 60u64))
    );

    assert_eq!(parse_inventory(b"10\n"), Ok((b"".as_slice(), 10u64)));

    assert_eq!(parse_inventory(b"10\n\n"), Ok((b"\n".as_slice(), 10u64)));
}

crate::test_day!(RUN, "day1", "67027", "197291");
