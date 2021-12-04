use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{flat_map, map, value};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;

use crate::{parsers, Day};
use std::cmp::Ordering;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(
        flat_map(terminated(BinaryLen::parse, parsers::newline), |first| {
            map(
                fold_many0(
                    terminated(parse_bin, parsers::newline),
                    move || {
                        let mut counts = Counts::new(first.len as usize);
                        counts.count(first.value);
                        counts
                    },
                    |mut counts, num| {
                        counts.count(num);
                        counts
                    },
                ),
                |counts| {
                    let (epsilon, gamma) = counts.epsilon_gamma();
                    log::debug!("epsilon: {}, gamma: {}", epsilon, gamma);
                    (epsilon * gamma) as i64
                },
            )
        }),
        input,
    )
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
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

                    (o2 * co2) as i64
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
    num_digits: usize,
    zeros: Vec<u32>,
    total: u32,
}

impl Counts {
    pub fn new(num_digits: usize) -> Self {
        Self {
            num_digits,
            zeros: vec![0; num_digits],
            total: 0,
        }
    }

    pub fn zeros_at(&self, index: usize) -> u32 {
        self.zeros[index]
    }

    pub fn ones_at(&self, index: usize) -> u32 {
        self.total - self.zeros[index]
    }

    pub fn count(&mut self, mut num: u32) {
        self.total += 1;
        for i in 0..self.num_digits {
            if num & 1 == 0 {
                self.zeros[i] += 1;
            }
            num >>= 1;
        }
    }

    pub fn uncount(&mut self, mut num: u32) {
        self.total -= 1;

        for i in 0..self.num_digits {
            if num & 1 == 0 {
                self.zeros[i] -= 1;
            }
            num >>= 1;
        }
    }

    pub fn epsilon_gamma(&self) -> (u64, u64) {
        let mut epsilon = 0;
        let mut gamma = 0;
        let mut mask = 1;
        for i in 0..self.num_digits {
            match self.most_common_bit(i) {
                Some(bit) => {
                    if bit {
                        gamma |= mask;
                    } else {
                        epsilon |= mask;
                    }
                }
                None => log::error!("tie at bit {}", i),
            }
            mask <<= 1;
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
        map(
            fold_many0(
                alt((value(0, tag(b"0")), value(1, tag(b"1")))),
                || (0, 0),
                |(len, value), digit| (len + 1, (value << 1) | digit),
            ),
            |(len, value)| BinaryLen { len, value },
        )(input)
    }
}

pub fn parse_bin_digit(input: &[u8]) -> IResult<&[u8], u32> {
    alt((value(0, tag(b"0")), value(1, tag(b"1"))))(input)
}

pub fn parse_bin(input: &[u8]) -> IResult<&[u8], u32> {
    fold_many0(
        alt((value(0, tag(b"0")), value(1, tag(b"1")))),
        || 0,
        |value, digit| (value << 1) | digit,
    )(input)
}

crate::test_day!(crate::day3::RUN, "day3", 4147524, 3570354);
