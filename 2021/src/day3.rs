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

    todo!("implement solution here");

    parser.finish()?;

    println!("{}", 0);
    Ok(())
}

struct Binary {
    digits: Vec<bool>,
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
}
