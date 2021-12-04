use crate::{Day, FileParser};
use std::cmp::Ordering;
use std::io::Read;
use std::str::FromStr;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);
    let mut counts = Counts::new();

    for num in parser.iter_parse::<Binary>() {
        counts.count(&num);
    }

    parser.finish()?;

    let (epsilon, gamma) = counts.epsilon_gamma();

    log::debug!("epsilon: {}, gamma: {}", epsilon, gamma);
    Ok((epsilon * gamma) as i64)
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);
    let nums: Vec<_> = parser.iter_parse::<Binary>().collect();

    parser.finish()?;

    let mut o2_candidates = nums.clone();
    let mut bit = 0;
    while o2_candidates.len() > 1 {
        let mut counts = Counts::new();
        for num in o2_candidates.iter() {
            counts.count(num);
        }
        let mcb = counts.most_common_bit(bit).unwrap_or(true);
        o2_candidates.retain(|num| num.bit(bit) == mcb);
        bit += 1;
    }

    let mut co2_candidates = nums;
    let mut bit = 0;
    while co2_candidates.len() > 1 {
        let mut counts = Counts::new();
        for num in co2_candidates.iter() {
            counts.count(num);
        }
        let lcb = !counts.most_common_bit(bit).unwrap_or(true);
        co2_candidates.retain(|num| num.bit(bit) == lcb);
        bit += 1;
    }

    log::debug!("{:?}", o2_candidates[0]);
    let o2 = o2_candidates[0].to_u64();
    let co2 = co2_candidates[0].to_u64();

    log::debug!("o2: {}, co2: {}", o2, co2);
    Ok((o2 * co2) as i64)
}

#[derive(Debug, Copy, Clone)]
struct Binary {
    value: u32,
    len: u32,
}

impl Binary {
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

impl FromStr for Binary {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut len = 0;
        let mut value = 0;
        for ch in s.chars() {
            len += 1;
            value <<= 1;
            if ch == '1' {
                value |= 1;
            } else if ch != '0' {
                return Err(())
            }
        }
        Ok(Binary { len, value })
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
            if ! d {
                self.zeros[i] += 1;
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
