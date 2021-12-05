use nom::{
    combinator::flat_map,
    multi::{fold_many0, fold_many_m_n},
    sequence::terminated,
};

use crate::{parsers, Day};
use std::{cmp::Ordering, collections::VecDeque};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    day1_impl(input, 1)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    day1_impl(input, 3)
}

fn day1_impl(input: &[u8], window_size: usize) -> anyhow::Result<i64> {
    parsers::parse(
        flat_map(
            // first fill the scanning buffer
            fold_many_m_n(
                window_size,
                window_size,
                terminated(parsers::i32, parsers::newline),
                Sonar::default,
                |mut acc, item| {
                    acc.push_init(item);
                    acc
                },
            ),
            // then switch to scanning mode
            |mut sonar| {
                fold_many0(
                    terminated(parsers::i32, parsers::newline),
                    || 0,
                    move |increases, item| {
                        if sonar.push_scan(item) == Ordering::Greater {
                            increases + 1
                        } else {
                            increases
                        }
                    },
                )
            },
        ),
        input,
    )
}

#[derive(Default)]
struct Sonar {
    window_sum: i32,
    buffer: VecDeque<i32>,
}

impl Sonar {
    fn push_init(&mut self, value: i32) {
        self.buffer.push_back(value);
        self.window_sum += value;
    }

    fn push_scan(&mut self, new_reading: i32) -> std::cmp::Ordering {
        let old_reading = self.buffer.pop_front().expect("window_size must be > 0");
        self.buffer.push_back(new_reading);
        let new_window_sum = self.window_sum + new_reading - old_reading;
        let result = new_window_sum.cmp(&self.window_sum);
        self.window_sum = new_window_sum;
        result
    }
}

crate::test_day!(crate::day1::RUN, "day1", 1527, 1575);
