#![allow(unused)]

use anyhow::bail;
use nom::{
    bytes::complete::take_while1, combinator::map, multi::fold_many0, sequence::terminated, IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let fuel = parsers::parse(
        fold_many0(
            terminated(parse_snafu_to_decimal, newline),
            || 0,
            |total, item| total + item,
        ),
        input,
    )?;

    let result = decimal_to_snafu(fuel);

    Ok(result)
}

fn decimal_to_snafu(fuel: i64) -> String {
    let mut out = Vec::new();
    let mut rest = fuel;
    while rest > 0 {
        let digit = rest % 5;
        rest /= 5;
        match digit {
            0 => out.push(b'0'),
            1 => out.push(b'1'),
            2 => out.push(b'2'),
            3 => {
                out.push(b'=');
                rest += 1;
            }
            4 => {
                out.push(b'-');
                rest += 1;
            }
            _ => unreachable!(),
        }
    }
    if out.is_empty() {
        out.push(b'0');
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    Ok("n/a".to_string())
}

fn parse_snafu_to_decimal(input: &[u8]) -> IResult<&[u8], i64> {
    map(
        take_while1(|ch| b"=-012".contains(&ch)),
        |snafu: &[u8]| {
            snafu
                .iter()
                .rev()
                .fold((1, 0), |(value, result), digit| {
                    (value * 5, result + snafu_digit(*digit) * value)
                })
                .1
        },
    )(input)
}

fn snafu_digit(digit: u8) -> i64 {
    match digit {
        b'=' => -2,
        b'-' => -1,
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        _ => panic!("invalid snafu digit"),
    }
}

fn digit_snafu(digit: i64) -> char {
    match digit {
        -2 => '=',
        -1 => '-',
        0 => '0',
        1 => '1',
        2 => '2',
        _ => panic!("invalid snafu digit"),
    }
}

crate::test_day!(RUN, "day25", "2-2--02=1---1200=0-1", "n/a");
