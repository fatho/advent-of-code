use nom::{
    character::complete::alpha1,
    combinator::map,
    multi::fold_many0,
    sequence::{terminated, tuple},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let sum = parsers::parse(
        fold_many0(
            terminated(parse_rucksack_halves, newline),
            || 0u32,
            |count, (h1, h2)| count + (h1 & h2).trailing_zeros(),
        ),
        input,
    )?;
    Ok(sum.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let sum = parsers::parse(
        fold_many0(
            tuple((
                terminated(parse_rucksack, newline),
                terminated(parse_rucksack, newline),
                terminated(parse_rucksack, newline),
            )),
            || 0u32,
            |count, (r1, r2, r3)| count + (r1 & r2 & r3).trailing_zeros(),
        ),
        input,
    )?;
    Ok(sum.to_string())
}

fn priority(item: u8) -> u8 {
    match item {
        b'a'..=b'z' => item - b'a' + 1,
        b'A'..=b'Z' => item - b'A' + 27,
        _ => panic!("not a valid item {item}"),
    }
}

fn parse_rucksack(input: &[u8]) -> IResult<&[u8], u64> {
    map(alpha1, items_to_bitset)(input)
}

fn parse_rucksack_halves(input: &[u8]) -> IResult<&[u8], (u64, u64)> {
    map(alpha1, |items: &[u8]| {
        let mid = items.len() / 2;
        let h1 = items_to_bitset(&items[0..mid]);
        let h2 = items_to_bitset(&items[mid..]);
        (h1, h2)
    })(input)
}

fn items_to_bitset(items: &[u8]) -> u64 {
    // 52 different item types conveniently fit into a single u64
    items
        .iter()
        .copied()
        .map(priority)
        .fold(0, |set, index| set | (1u64 << index))
}

crate::test_day!(RUN, "day3", "8105", "2363");
