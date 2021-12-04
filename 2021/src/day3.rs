use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::multi::{fold_many0, many0};
use nom::sequence::terminated;
use nom::IResult;

use crate::{parsers, Day};
use std::cmp::Ordering;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(
        map(
            fold_many0(
                terminated(Binary::parse, parsers::newline),
                || Counts::new(),
                |mut counts, num| {
                    counts.count(&num);
                    counts
                },
            ),
            |counts| {
                let (epsilon, gamma) = counts.epsilon_gamma();
                log::debug!("epsilon: {}, gamma: {}", epsilon, gamma);
                (epsilon * gamma) as i64
            },
        ),
        input,
    )
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(
        map(many0(terminated(Binary::parse, parsers::newline)), |nums| {
            // count bits of all numbers
            let mut counts = Counts::new();
            for num in nums.iter() {
                counts.count(num);
            }
            // determine o2 number
            let mut o2_candidates = nums.clone();
            {
                let mut o2_counts = counts.clone();
                let mut bit = 0;
                while o2_candidates.len() > 1 {
                    let mcb = o2_counts.most_common_bit(bit).unwrap_or(true);
                    o2_candidates.retain(|num| {
                        let keep = num.bit(bit) == mcb;
                        if ! keep {
                            o2_counts.uncount(num);
                        }
                        keep
                    });
                    bit += 1;
                }
            }

            let mut co2_candidates = nums;
            {
                let mut co2_counts = counts.clone();
                let mut bit = 0;
                while co2_candidates.len() > 1 {
                    let lcb = ! co2_counts.most_common_bit(bit).unwrap_or(true);
                    co2_candidates.retain(|num| {
                        let keep = num.bit(bit) == lcb;
                        if ! keep {
                            co2_counts.uncount(num);
                        }
                        keep
                    });
                    bit += 1;
                }
            }

            log::debug!("{:?}", o2_candidates[0]);
            let o2 = o2_candidates[0].to_u64();
            let co2 = co2_candidates[0].to_u64();

            log::debug!("o2: {}, co2: {}", o2, co2);
            (o2 * co2) as i64
        }),
        input,
    )
}

#[derive(Debug, Copy, Clone)]
struct Binary {
    value: u32,
    len: u32,
}

impl Binary {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Binary> {
        map(
            fold_many0(
                alt((value(0, tag(b"0")), value(1, tag(b"1")))),
                || (0, 0),
                |(len, value), digit| {
                    (
                        len + 1,
                        (value << 1) | digit,
                    )
                },
            ),
            |(len, value)| Binary { len, value },
        )(input)
    }

    pub fn bit(self, index: usize) -> bool {
        self.value & (1 << (self.len - 1 - index as u32)) != 0
    }

    pub fn to_u64(&self) -> u64 {
        self.value as u64
    }

    pub fn digits(&self) -> DigitIter {
        DigitIter {
            index: self.len,
            value: self.value,
        }
    }
}
struct DigitIter {
    index: u32,
    value: u32,
}

impl Iterator for DigitIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > 0 {
            self.index -= 1;
            Some(self.value & (1 << self.index) != 0)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Counts {
    num_digits: usize,
    zeros: Vec<u32>,
    total: u32,
}

impl Counts {
    pub fn new() -> Self {
        Self {
            num_digits: 0,
            zeros: Vec::new(),
            total: 0,
        }
    }

    pub fn zeros_at(&self, index: usize) -> u32 {
        self.zeros[index]
    }

    pub fn ones_at(&self, index: usize) -> u32 {
        self.total - self.zeros[index]
    }

    pub fn count(&mut self, num: &Binary) {
        let num_digits = num.len as usize;
        self.num_digits = num_digits.max(self.num_digits);
        self.total += 1;
        self.zeros.resize(self.num_digits, 0);

        for (i, d) in num.digits().enumerate() {
            if !d {
                self.zeros[i] += 1;
            }
        }
    }

    pub fn uncount(&mut self, num: &Binary) {
        self.total -= 1;

        for (i, d) in num.digits().enumerate() {
            if !d {
                self.zeros[i] -= 1;
            }
        }
    }

    pub fn epsilon_gamma(&self) -> (u64, u64) {
        let mut epsilon = 0;
        let mut gamma = 0;
        for i in 0..self.num_digits {
            epsilon *= 2;
            gamma *= 2;
            match self.most_common_bit(i) {
                Some(bit) => {
                    if bit {
                        gamma += 1;
                    } else {
                        epsilon += 1;
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

crate::test_day!(crate::day3::RUN, "day3", 4147524, 3570354);
