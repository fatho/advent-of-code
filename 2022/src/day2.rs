use nom::{
    bytes::complete::tag,
    character::complete::newline,
    multi::fold_many0,
    sequence::{separated_pair, terminated},
};

use crate::{
    parsers::{self, byte_range},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    shared(input, &SCORE_PART1)
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    shared(input, &SCORE_PART2)
}

pub fn shared(input: &[u8], table: &[[u32; 3]; 3]) -> anyhow::Result<String> {
    let total_score = parsers::parse(
        fold_many0(
            terminated(
                separated_pair(byte_range(b'A'..=b'C'), tag(" "), byte_range(b'X'..=b'Z')),
                newline,
            ),
            || 0,
            |acc, (opponent, me)| acc + table[(opponent - b'A') as usize][(me - b'X') as usize],
        ),
        input,
    )?;
    Ok(total_score.to_string())
}

// A, X -> Rock
// B, Y -> Paper
// C, Z -> Scissors

const SCORE_PART1: [[u32; 3]; 3] = [
    /*
    [X, Y, Z]  */
    [4, 8, 3], // A
    [1, 5, 9], // B
    [7, 2, 6], // C
];

// X -> Loss
// Y -> Draw
// Z -> Win

const SCORE_PART2: [[u32; 3]; 3] = [
    /*
    [X, Y, Z]  */
    [3, 4, 8], // A
    [1, 5, 9], // B
    [2, 6, 7], // C
];

crate::test_day!(RUN, "day2", "11666", "12767");
