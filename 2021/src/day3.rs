use crate::FileParser;
use std::cmp::Ordering;
use std::io::Read;
use std::str::FromStr;

pub fn part1(input: &mut dyn Read) -> anyhow::Result<()> {
    let mut parser = FileParser::new(input);
    let mut counts = Counts::new();

    for num in parser.iter_parse::<Binary>() {
        counts.count(&num);
    }

    parser.finish()?;

    let (epsilon, gamma) = counts.epsilon_gamma();

    eprintln!("epsilon: {}, gamma: {}", epsilon, gamma);
    println!("{}", epsilon * gamma);
    Ok(())
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<()> {
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
        o2_candidates.retain(|num| num.digits[bit] == mcb);
        bit += 1;
    }

    let mut co2_candidates = nums.clone();
    let mut bit = 0;
    while co2_candidates.len() > 1 {
        let mut counts = Counts::new();
        for num in co2_candidates.iter() {
            counts.count(num);
        }
        let lcb = !counts.most_common_bit(bit).unwrap_or(true);
        co2_candidates.retain(|num| num.digits[bit] == lcb);
        bit += 1;
    }

    eprintln!("{:?}", o2_candidates[0]);
    let o2 = o2_candidates[0].to_u64();
    let co2 = co2_candidates[0].to_u64();

    eprintln!("o2: {}, co2: {}", o2, co2);
    println!("{}", o2 * co2);
    Ok(())
}

#[derive(Debug, Clone)]
struct Binary {
    digits: Vec<bool>,
}

impl Binary {
    pub fn to_u64(&self) -> u64 {
        let mut acc = 0;
        for d in self.digits.iter() {
            acc *= 2;
            if *d {
                acc += 1;
            }
        }
        acc
    }
}

impl FromStr for Binary {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut digits = Vec::new();
        for ch in s.chars() {
            match ch {
                '0' => digits.push(false),
                '1' => digits.push(true),
                _ => return Err(()),
            }
        }
        Ok(Binary { digits })
    }
}

struct Counts {
    num_digits: usize,
    zeros: Vec<u32>,
    ones: Vec<u32>,
}

impl Counts {
    pub fn new() -> Self {
        Self {
            num_digits: 0,
            ones: Vec::new(),
            zeros: Vec::new(),
        }
    }

    pub fn count(&mut self, num: &Binary) {
        let num_digits = num.digits.len();
        self.num_digits = num_digits.max(self.num_digits);
        self.zeros.resize(self.num_digits, 0);
        self.ones.resize(self.num_digits, 0);

        for (i, d) in num.digits.iter().enumerate() {
            if *d {
                self.ones[i] += 1;
            } else {
                self.zeros[i] += 1;
            }
        }
    }

    pub fn epsilon_gamma(&self) -> (u64, u64) {
        let mut epsilon = 0;
        let mut gamma = 0;
        for (i, (c0, c1)) in self.zeros.iter().zip(self.ones.iter()).enumerate() {
            epsilon *= 2;
            gamma *= 2;
            if c0 > c1 {
                epsilon += 1;
            } else if c0 < c1 {
                gamma += 1;
            } else {
                eprintln!("tie at bit {}", i);
            }
        }
        (epsilon, gamma)
    }

    pub fn most_common_bit(&self, index: usize) -> Option<bool> {
        match self.zeros[index].cmp(&self.ones[index]) {
            Ordering::Less => Some(true),
            Ordering::Equal => None,
            Ordering::Greater => Some(false),
        }
    }
}
