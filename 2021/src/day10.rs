#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

// TODO: share some more code between parts

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut stack = Vec::new();
    let mut total_score = 0;
    for line in input.split(|b| *b == b'\n') {
        // just for skipping last line
        if line.is_empty() {
            continue;
        }

        stack.clear();

        for b in line.iter().copied() {
            if matches!(b, b'(' | b'[' | b'<' | b'{') {
                stack.push(b);
            }
            if matches!(b, b')' | b']' | b'>' | b'}') {
                if let Some(expected) = stack.pop() {
                    if closing(expected) != b {
                        // corrupted!
                        total_score += corruption_score(b);
                        break;
                    }
                } else {
                    // Too many closing characters, is this a possible failure mode?
                    unreachable!()
                }
            }
        }
        // may or may not be incomplete
    }
    Ok(format!("{}", total_score))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut stack = Vec::new();
    let mut total_scores = Vec::new();
    for line in input.split(|b| *b == b'\n') {
        // just for skipping last line
        if line.is_empty() {
            continue;
        }

        stack.clear();

        let mut corrupted = false;
        for b in line.iter().copied() {
            if matches!(b, b'(' | b'[' | b'<' | b'{') {
                stack.push(b);
            }
            if matches!(b, b')' | b']' | b'>' | b'}') {
                if let Some(expected) = stack.pop() {
                    if closing(expected) != b {
                        corrupted = true;
                        break;
                    }
                } else {
                    // Too many closing characters, is this a possible failure mode?
                    unreachable!()
                }
            }
        }
        if !corrupted && !stack.is_empty() {
            let line_score = stack
                .drain(..)
                .rev()
                .fold(0, |score, open| score * 5 + completion_score(closing(open)));
            total_scores.push(line_score);
        }
    }
    total_scores.sort_unstable();
    let final_score = total_scores[total_scores.len() / 2];
    Ok(format!("{}", final_score))
}

fn closing(opening: u8) -> u8 {
    match opening {
        b'(' => b')',
        b'[' => b']',
        b'<' => b'>',
        b'{' => b'}',
        _ => panic!("not a paren"),
    }
}

fn corruption_score(closing: u8) -> i64 {
    match closing {
        b')' => 3,
        b']' => 57,
        b'>' => 25137,
        b'}' => 1197,
        _ => panic!("not a paren"),
    }
}

fn completion_score(closing: u8) -> i64 {
    match closing {
        b')' => 1,
        b']' => 2,
        b'}' => 3,
        b'>' => 4,
        _ => panic!("not a paren"),
    }
}
crate::test_day!(crate::day10::RUN, "day10", "315693", "1870887234");
