#![allow(unused)]

use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::map,
    multi::fold_many0,
    sequence::{separated_pair, terminated},
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let total_score = parsers::parse(
        fold_many0(
            terminated(
                separated_pair(parse_col1, tag(" "), parse_col2_part1),
                newline,
            ),
            || 0,
            |acc, (opponent, me)| acc + Round { opponent, me }.score(),
        ),
        input,
    )?;
    Ok(total_score.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let total_score = parsers::parse(
        fold_many0(
            terminated(
                separated_pair(parse_col1, tag(" "), parse_col2_part2),
                newline,
            ),
            || 0,
            |acc, (opponent, outcome)| {
                acc + Round {
                    opponent,
                    me: Sign::for_outcome_against(outcome, opponent),
                }
                .score()
            },
        ),
        input,
    )?;
    Ok(total_score.to_string())
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Sign {
    Rock,
    Paper,
    Scissors,
}

impl Sign {
    fn defeats(self, other: Sign) -> bool {
        use Sign::*;

        matches!(
            (self, other),
            (Rock, Scissors) | (Scissors, Paper) | (Paper, Rock)
        )
    }

    fn score(self) -> u32 {
        match self {
            Sign::Rock => 1,
            Sign::Paper => 2,
            Sign::Scissors => 3,
        }
    }

    fn for_outcome_against(outcome: Outcome, other: Sign) -> Sign {
        match outcome {
            Outcome::Win => match other {
                Sign::Rock => Sign::Paper,
                Sign::Paper => Sign::Scissors,
                Sign::Scissors => Sign::Rock,
            },
            Outcome::Draw => other,
            Outcome::Loss => match other {
                Sign::Rock => Sign::Scissors,
                Sign::Paper => Sign::Rock,
                Sign::Scissors => Sign::Paper,
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Round {
    opponent: Sign,
    me: Sign,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Outcome {
    fn score(&self) -> u32 {
        match self {
            Outcome::Win => 6,
            Outcome::Draw => 3,
            Outcome::Loss => 0,
        }
    }
}

impl Round {
    fn score(&self) -> u32 {
        self.me.score() + self.outcome().score()
    }

    fn outcome(&self) -> Outcome {
        if self.opponent == self.me {
            Outcome::Draw
        } else if self.opponent.defeats(self.me) {
            Outcome::Loss
        } else {
            assert!(self.me.defeats(self.opponent));
            Outcome::Win
        }
    }
}

fn parse_col1(input: &[u8]) -> nom::IResult<&[u8], Sign> {
    alt((
        map(tag("A"), |_| Sign::Rock),
        map(tag("B"), |_| Sign::Paper),
        map(tag("C"), |_| Sign::Scissors),
    ))(input)
}

fn parse_col2_part1(input: &[u8]) -> nom::IResult<&[u8], Sign> {
    alt((
        map(tag("X"), |_| Sign::Rock),
        map(tag("Y"), |_| Sign::Paper),
        map(tag("Z"), |_| Sign::Scissors),
    ))(input)
}

fn parse_col2_part2(input: &[u8]) -> nom::IResult<&[u8], Outcome> {
    alt((
        map(tag("X"), |_| Outcome::Loss),
        map(tag("Y"), |_| Outcome::Draw),
        map(tag("Z"), |_| Outcome::Win),
    ))(input)
}

crate::test_day!(RUN, "day2", "11666", "12767");
