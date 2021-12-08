#![allow(unused_imports)]

use std::fmt::Display;

use crate::{parsers, Day};
use anyhow::Context;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{all_consuming, flat_map, map, map_opt};
use nom::multi::{fold_many0, fold_many1, many0, many_m_n};
use nom::sequence::{preceded, separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let entries = parsers::parse(many0(terminated(p_entry, parsers::newline)), input)?;

    // 1: 2 segments
    // 4: 4 segments
    // 7: 3 segments
    // 8: 7 segments

    let num_unique = entries
        .iter()
        .flat_map(|e| &e.outputs)
        .filter(|p| matches!(p.count_set(), 2 | 3 | 4 | 7))
        .count();

    Ok(num_unique as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let entries = parsers::parse(many0(terminated(p_entry, parsers::newline)), input)?;

    let mut result = 0;
    for e in entries {
        let (_, digits) = Mapping::deduct_rec(&e.observations).context("invalid observation")?;
        let mut num = 0;
        for o in &e.outputs {
            num *= 10;
            let d = digits
                .iter()
                .position(|p| p == o)
                .context("invalid pattern")?;
            num += d;
        }
        result += num;
    }

    Ok(result as i64)
}

fn p_pattern(input: &[u8]) -> IResult<&[u8], Pattern> {
    map_opt(
        take_while(|c| matches!(c, b'a'..=b'g')),
        |patchars: &[u8]| {
            if patchars.is_empty() {
                None
            } else {
                let bits = patchars
                    .iter()
                    .map(|x| x - b'a')
                    .fold(0, |acc, seg| acc | (1 << seg));
                Some(Pattern::new(bits))
            }
        },
    )(input)
}

fn p_entry(input: &[u8]) -> IResult<&[u8], Entry> {
    map(
        separated_pair(
            many_m_n(10, 10, terminated(p_pattern, tag(" "))),
            tag("|"),
            many_m_n(4, 4, preceded(tag(" "), p_pattern)),
        ),
        |(observations, outputs)| Entry {
            observations,
            outputs,
        },
    )(input)
}

#[derive(Debug, Clone, Copy)]
struct Mapping {
    /// For each of the orignal a-g (0-6) segments there is a pattern indicating
    /// which of the permuted segments could correspond to it.
    segments: u64,
}

impl Mapping {
    fn new() -> Self {
        let all = Pattern::all().bits as u64;
        Mapping {
            segments: all | all << 8 | all << 16 | all << 24 | all << 32 | all << 40 | all << 48,
        }
    }

    fn deduct_rec(obs: &[Pattern]) -> Option<(Self, Vec<Pattern>)> {
        fn go(
            state: Mapping,
            obs: &[Pattern],
            digits: &mut [Option<Pattern>],
        ) -> Option<(Mapping, Vec<Pattern>)> {
            if obs.is_empty() {
                let valid = (0..7)
                    .map(|seg| state.segment(seg))
                    .all(|pat| pat.count_set() == 1);
                if valid {
                    Some((state, digits.iter().map(|d| d.unwrap()).collect()))
                } else {
                    None
                }
            } else {
                let choices = COUNT_TO_DIGITS[obs[0].count_set() as usize];
                for choice in choices {
                    let digit_index = *choice as usize;
                    if digits[digit_index].is_none() {
                        digits[digit_index] = Some(obs[0]);
                        if let Some(result) = go(state.restrict(*choice, obs[0]), &obs[1..], digits)
                        {
                            return Some(result);
                        }
                        digits[digit_index] = None;
                    }
                }
                None
            }
        }

        let mut remaining = vec![None; 10];
        let state = Mapping::new();
        go(state, obs, &mut remaining)
    }

    fn restrict(self, digit: u8, pat: Pattern) -> Self {
        let on_segs = DIGIT_TO_SEGMENTS[digit as usize];
        let mut out = self;
        // restrict each of the segments to the current pattern
        for seg in 0..7 {
            let offset = seg * 8;

            // If the segment is on for this digit...
            let bits = if on_segs.contains(&seg) {
                // we know it must be one of this digit's pattern
                pat.bits
            } else {
                // otherwise, we know it's definitely not one of this digit's
                // pattern
                !pat.bits
            };

            let mask = !(0xFF << offset) | ((bits as u64) << offset);

            out.segments &= mask;
        }

        out
    }

    fn segment(self, index: u8) -> Pattern {
        Pattern::new((self.segments >> (index * 8)) as u8)
    }
}

#[test]
#[allow(clippy::unusual_byte_groupings)]
fn test_restrict() {
    let map = Mapping::new();
    let pat = Pattern::from_segs(&[3, 6]);
    let map2 = map.restrict(1, pat);
    assert_eq!(
        map2.segments,
        0b00000000_00110111_01001000_00110111_00110111_01001000_00110111_00110111_u64,
        "\n{:b}\n{:b}",
        map2.segments,
        0b00000000_00110111_01001000_00110111_00110111_01001000_00110111_00110111_u64
    );
    assert_eq!(map2.segment(2), pat);
    assert_eq!(map2.segment(5), pat);
}

const COUNT_TO_DIGITS: &[&[u8]] = &[
    // 0
    &[],
    // 1
    &[],
    // 2
    &[1],
    // 3
    &[7],
    // 4
    &[4],
    // 5
    &[2, 3, 5],
    // 6
    &[0, 6, 9],
    // 7
    &[8],
];

//     // a b c d e f g
//     // 0 1 2 3 4 5 6
const DIGIT_TO_SEGMENTS: &[&[u8]] = &[
    // 0
    &[0, 1, 2, 4, 5, 6],
    // 1
    &[2, 5],
    // 2
    &[0, 2, 3, 4, 6],
    // 3
    &[0, 2, 3, 5, 6],
    // 4
    &[1, 2, 3, 5],
    // 5
    &[0, 1, 3, 5, 6],
    // 6
    &[0, 1, 3, 4, 5, 6],
    // 7
    &[0, 2, 5],
    // 8
    &[0, 1, 2, 3, 4, 5, 6],
    // 9
    &[0, 1, 2, 3, 5, 6],
];

#[derive(Debug)]
struct Entry {
    observations: Vec<Pattern>,
    outputs: Vec<Pattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

struct Pattern {
    /// segments lighting up in bit 0-6 corresponding to sgements a-g
    bits: u8,
}

impl Pattern {
    pub const fn new(bits: u8) -> Self {
        Pattern { bits }
    }

    pub const fn all() -> Self {
        Self::new(0b1111111)
    }

    #[cfg(test)]
    pub fn from_segs(segs: &[u8]) -> Self {
        let mut bits = 0;
        for seg in segs {
            bits |= 1 << *seg;
        }
        Self::new(bits)
    }

    pub const fn count_set(self) -> u32 {
        self.bits.count_ones()
    }

    pub fn seg(self, index: u8) -> bool {
        (self.bits & (1 << index)) != 0
    }
}

impl Display for Pattern {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = if self.seg(0) { 'a' } else { '.' };
        let b = if self.seg(1) { 'b' } else { '.' };
        let c = if self.seg(2) { 'c' } else { '.' };
        let d = if self.seg(3) { 'd' } else { '.' };
        let e = if self.seg(4) { 'e' } else { '.' };
        let f = if self.seg(5) { 'f' } else { '.' };
        let g = if self.seg(6) { 'g' } else { '.' };

        use std::fmt::Write;

        writeln!(fmt, " {}{}{}{} ", a, a, a, a)?;
        writeln!(fmt, "{}    {}", b, c)?;
        writeln!(fmt, "{}    {}", b, c)?;
        writeln!(fmt, " {}{}{}{} ", d, d, d, d)?;
        writeln!(fmt, "{}    {}", e, f)?;
        writeln!(fmt, "{}    {}", e, f)?;
        writeln!(fmt, " {}{}{}{} ", g, g, g, g)?;
        Ok(())
    }
}

crate::test_day!(crate::day8::RUN, "day8", 278, 986179);
