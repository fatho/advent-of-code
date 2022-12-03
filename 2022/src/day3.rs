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
    let mut buf = input.to_owned();
    let mut sum = 0u32;
    // Sort ranges
    for (index, line) in buf.split_mut(|ch| *ch == b'\n').enumerate() {
        if line.len() & 1 == 1 {
            bail!("Rucksack with odd number of items on line {}", index + 1);
        } else if line.is_empty() {
            continue;
        }

        let mid = line.len() / 2;
        line[0..mid].sort();
        line[mid..].sort();

        if let Some(misplaced) = common_items(&line[0..mid], &line[mid..]).copied().next() {
            sum += priority(misplaced) as u32;
        } else {
            bail!("No misplaced item on line {}", index + 1);
        }
    }
    Ok(sum.to_string())
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

fn common_items<'a, T>(sorted_fst: &'a [T], sorted_snd: &'a [T]) -> CommonItems<'a, T> {
    CommonItems {
        fst: sorted_fst,
        snd: sorted_snd,
        ifst: 0,
        isnd: 0,
    }
}

struct CommonItems<'a, T> {
    fst: &'a [T],
    snd: &'a [T],
    ifst: usize,
    isnd: usize,
}

impl<'a, T: Ord> Iterator for CommonItems<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.ifst < self.fst.len() && self.isnd < self.snd.len() {
            match self.fst[self.ifst].cmp(&self.snd[self.isnd]) {
                std::cmp::Ordering::Less => {
                    self.ifst += 1;
                }
                std::cmp::Ordering::Equal => {
                    let the_item = &self.fst[self.ifst];

                    // Skip items of this type
                    while self.ifst < self.fst.len() && &self.fst[self.ifst] == the_item {
                        self.ifst += 1;
                    }
                    while self.isnd < self.snd.len() && &self.snd[self.isnd] == the_item {
                        self.isnd += 1;
                    }

                    return Some(the_item);
                }
                std::cmp::Ordering::Greater => {
                    self.isnd += 1;
                }
            }
        }
        // As soon as one half is exhausted, there can no longer be more common items
        None
    }
}

#[test]
fn test_common_items() {
    assert_eq!(
        common_items(&[1, 2, 2, 4, 5, 6, 6, 8, 8], &[2, 3, 4, 4, 6, 6, 7, 7])
            .copied()
            .collect::<Vec<u8>>(),
        vec![2, 4, 6]
    );
}

fn priority(item: u8) -> u8 {
    match item {
        b'a'..=b'z' => item - b'a' + 1,
        b'A'..=b'Z' => item - b'A' + 27,
        _ => panic!("not a valid item {item}"),
    }
}

crate::test_day!(RUN, "day3", "8105", "2363");
