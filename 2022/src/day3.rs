#![allow(unused)]

use std::collections::HashSet;

use anyhow::bail;
use nom::{
    character::complete::alpha1,
    combinator::flat_map,
    multi::{fold_many0, fold_many_m_n},
    sequence::terminated,
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let prios = parsers::parse(
        fold_many0(
            terminated(alpha1, newline),
            || 0,
            |sum, rucksack| {
                debug_assert!(rucksack.len() % 2 == 0);
                let mid = rucksack.len() / 2;
                let first: HashSet<u8> = rucksack[0..mid].iter().copied().collect();

                let misplaced_item = rucksack[mid..]
                    .iter()
                    .copied()
                    .find(|item| first.contains(item))
                    .unwrap();

                sum + priority(misplaced_item) as u32
            },
        ),
        input,
    )?;
    Ok(prios.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let prios = parsers::parse(
        fold_many0(
            flat_map(
                // Parse first rucksack in group
                terminated(alpha1, newline),
                |rucksack| {
                    let item_types: HashSet<u8> = rucksack.iter().copied().collect();
                    fold_many_m_n(
                        2,
                        2,
                        terminated(alpha1, newline),
                        move || item_types.clone(),
                        |prev, rucksack| {
                            rucksack
                                .iter()
                                .copied()
                                .filter(|item| prev.contains(item))
                                .collect()
                        },
                    )
                },
            ),
            || 0,
            |sum, common_items| {
                debug_assert_eq!(common_items.len(), 1);
                let badge_item = common_items.into_iter().next().unwrap();
                sum + priority(badge_item) as u32
            },
        ),
        input,
    )?;
    Ok(prios.to_string())
}

fn priority(item: u8) -> u8 {
    match item {
        b'a'..=b'z' => item - b'a' + 1,
        b'A'..=b'Z' => item - b'A' + 27,
        _ => panic!("not a valid item {item}"),
    }
}

crate::test_day!(RUN, "day3", "8105", "2363");
