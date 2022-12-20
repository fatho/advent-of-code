#![allow(unused)]

use std::ops::Mul;

use anyhow::{bail, Context};
use nom::{multi::many0, sequence::terminated};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let nums = parsers::parse(
        many0(terminated(nom::character::complete::i64, newline)),
        input,
    )?;
    let mut seq = Seq::new(nums);

    // Move each number in order
    for num in 0..seq.nums.len() {
        let mut offset = seq.nums[num];

        match offset.cmp(&0) {
            std::cmp::Ordering::Less => {
                while offset < 0 {
                    offset += 1;
                    seq.move_left(num);
                }
            }
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => {
                while offset > 0 {
                    offset -= 1;
                    seq.move_right(num);
                }
            }
        }
    }

    let zero_num = seq.nums.iter().position(|n| *n == 0).context("need zero")?;
    let zero_pos = seq.num_to_pos[zero_num];

    let ret = [1000, 2000, 3000]
        .into_iter()
        .map(|offset| (zero_pos + offset) % seq.nums.len())
        .map(|pos| seq.nums[seq.pos_to_num[pos]])
        .sum::<i64>();

    Ok(ret.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut nums = parsers::parse(
        many0(terminated(nom::character::complete::i64, newline)),
        input,
    )?;
    let key = 811589153;
    for num in nums.iter_mut() {
        *num *= key;
    }

    let mut seq = Seq::new(nums);

    for i in 0..10 {
        // Move each number in order
        for num in 0..seq.nums.len() {
            let mut offset = seq.nums[num] % (seq.nums.len() as i64 - 1);

            match offset.cmp(&0) {
                std::cmp::Ordering::Less => {
                    while offset < 0 {
                        offset += 1;
                        seq.move_left(num);
                    }
                }
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => {
                    while offset > 0 {
                        offset -= 1;
                        seq.move_right(num);
                    }
                }
            }
        }
    }

    let zero_num = seq.nums.iter().position(|n| *n == 0).context("need zero")?;
    let zero_pos = seq.num_to_pos[zero_num];

    let ret = [1000, 2000, 3000]
        .into_iter()
        .map(|offset| (zero_pos + offset) % seq.nums.len())
        .map(|pos| seq.nums[seq.pos_to_num[pos]])
        .sum::<i64>();

    Ok(ret.to_string())
}

struct Seq {
    nums: Vec<i64>,
    num_to_pos: Vec<usize>,
    pos_to_num: Vec<usize>,
}

// TODO: optimize sequence representation and operations on it - there got to be a faster way
impl Seq {
    fn new(nums: Vec<i64>) -> Self {
        Self {
            num_to_pos: (0..nums.len()).collect(),
            pos_to_num: (0..nums.len()).collect(),
            nums,
        }
    }

    fn move_left(&mut self, num: usize) {
        let cur_pos = self.num_to_pos[num];

        let left_pos = if cur_pos == 0 {
            self.nums.len() - 1
        } else {
            cur_pos - 1
        };

        let left_num = self.pos_to_num[left_pos];

        self.pos_to_num[left_pos] = num;
        self.pos_to_num[cur_pos] = left_num;
        self.num_to_pos[left_num] = cur_pos;
        self.num_to_pos[num] = left_pos;
    }

    fn move_right(&mut self, num: usize) {
        let cur_pos = self.num_to_pos[num];

        let right_pos = if cur_pos == self.nums.len() - 1 {
            0
        } else {
            cur_pos + 1
        };

        let right_num = self.pos_to_num[right_pos];

        self.pos_to_num[right_pos] = num;
        self.pos_to_num[cur_pos] = right_num;
        self.num_to_pos[right_num] = cur_pos;
        self.num_to_pos[num] = right_pos;
    }
}

crate::test_day!(RUN, "day20", "23321", "1428396909280");
