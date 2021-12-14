use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;

use crate::{parsers, Day};
use std::cmp::Ordering;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    parsers::parse(
        flat_map(
            map(
                terminated(take_while(|b| b == b'0' || b == b'1'), parsers::newline),
                |digits| Counts {
                    ones: digits.iter().map(|d| (d - b'0') as u32).collect(),
                    total: 1,
                },
            ),
            |counts| {
                map(
                    fold_many0(
                        terminated(take_while(|b| b == b'0' || b == b'1'), parsers::newline),
                        move || counts.clone(),
                        |mut counts, bin| {
                            for (i, d) in bin.iter().enumerate() {
                                counts.ones[i] += (d - b'0') as u32;
                            }
                            counts.total += 1;
                            counts
                        },
                    ),
                    |counts| {
                        let (epsilon, gamma) = counts.epsilon_gamma();
                        format!("{}", epsilon * gamma)
                    },
                )
            },
        ),
        input,
    )
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    parsers::parse(
        flat_map(terminated(BinaryLen::parse, parsers::newline), |first| {
            map(
                fold_many0(
                    terminated(parse_bin, parsers::newline),
                    move || vec![first.value],
                    |mut nums, num| {
                        nums.push(num);
                        nums
                    },
                ),
                move |nums| {
                    // determine o2 number
                    let mut o2_candidates = nums.clone();
                    let mut co2_candidates = nums;

                    prune_candidates(&mut o2_candidates, first.len, true);
                    prune_candidates(&mut co2_candidates, first.len, false);

                    let o2 = o2_candidates[0];
                    let co2 = co2_candidates[0];

                    format!("{}", o2 * co2)
                },
            )
        }),
        input,
    )
}

fn prune_candidates(candidates: &mut Vec<u32>, num_bits: u32, most: bool) {
    let mut bit = num_bits;
    while candidates.len() > 1 {
        bit -= 1;

        let zeros: u32 = candidates
            .iter()
            .map(|x| (x & (1 << bit) == 0) as u32)
            .sum();

        let most_common = zeros <= (candidates.len() as u32 - zeros);

        let keep_when = most_common == most;
        candidates.retain(|num| (num & (1 << bit) != 0) == keep_when);
    }
}

#[derive(Debug, Clone, Default)]
struct Counts {
    ones: Vec<u32>,
    total: u32,
}

impl Counts {
    pub fn zeros_at(&self, index: usize) -> u32 {
        self.total - self.ones[index]
    }

    pub fn ones_at(&self, index: usize) -> u32 {
        self.ones[index]
    }

    pub fn epsilon_gamma(&self) -> (u64, u64) {
        let mut epsilon = 0;
        let mut gamma = 0;
        for i in 0..self.ones.len() {
            epsilon <<= 1;
            gamma <<= 1;
            match self.most_common_bit(i) {
                Some(bit) => {
                    if bit {
                        gamma |= 1;
                    } else {
                        epsilon |= 1;
                    }
                }
                None => log::error!("tie at bit {}", i),
            }
        }
        (epsilon, gamma)
    }

    pub fn most_common_bit(&self, index: usize) -> Option<bool> {
        match self.zeros_at(index).cmp(&self.ones_at(index)) {
            Ordering::Less => Some(true),
            Ordering::Equal => None,
            Ordering::Greater => Some(false),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct BinaryLen {
    value: u32,
    len: u32,
}

impl BinaryLen {
    pub fn parse(input: &[u8]) -> IResult<&[u8], BinaryLen> {
        map(take_while(|b| b == b'0' || b == b'1'), |digits: &[u8]| {
            BinaryLen {
                len: digits.len() as u32,
                value: digits
                    .iter()
                    .map(|d| d - b'0')
                    .fold(0, |acc, digit| acc << 1 | digit as u32),
            }
        })(input)
    }
}

pub fn parse_bin(input: &[u8]) -> IResult<&[u8], u32> {
    map(take_while(|b| b == b'0' || b == b'1'), |digits: &[u8]| {
        digits
            .iter()
            .map(|d| d - b'0')
            .fold(0, |acc, digit| acc << 1 | digit as u32)
    })(input)
}

crate::test_day!(crate::day3::RUN, "day3", "4147524", "3570354");
