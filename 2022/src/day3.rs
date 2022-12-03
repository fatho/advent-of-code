#![allow(unused)]

use std::collections::HashSet;

use anyhow::{anyhow, bail};

use crate::Day;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut buf = input.to_owned();
    let mut sum = 0u32;
    for (index, line) in buf.split_mut(|ch| *ch == b'\n').enumerate() {
        if line.len() & 1 == 1 {
            bail!("Rucksack with odd number of items on line {}", index + 1);
        } else if line.is_empty() {
            break;
        }

        let mid = line.len() / 2;

        if let Some(misplaced) = line[mid..]
            .iter()
            .copied()
            .find(|item| line[0..mid].contains(item))
        {
            sum += priority(misplaced) as u32;
        } else {
            bail!("No misplaced item on line {}", index + 1);
        }
    }
    Ok(sum.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut buf = input.to_owned();
    let mut sum = 0u32;

    let mut lines = buf.split_mut(|ch| *ch == b'\n').enumerate();
    while let Some((index, a)) = lines.next() {
        if a.is_empty() {
            break;
        }

        let (_, b) = lines
            .next()
            .ok_or_else(|| anyhow!("Incomplete group at line {}", index + 2))?;
        let (_, c) = lines
            .next()
            .ok_or_else(|| anyhow!("Incomplete group at line {}", index + 3))?;

        if let Some(badge) = c
            .iter()
            .copied()
            .find(|item| a.contains(item) && b.contains(item))
        {
            sum += priority(badge) as u32;
        } else {
            bail!("No badge in group starting on line {}", index + 1);
        }
    }
    Ok(sum.to_string())
}

fn priority(item: u8) -> u8 {
    match item {
        b'a'..=b'z' => item - b'a' + 1,
        b'A'..=b'Z' => item - b'A' + 27,
        _ => panic!("not a valid item {item}"),
    }
}

crate::test_day!(RUN, "day3", "8105", "2363");
