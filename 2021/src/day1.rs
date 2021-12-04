use anyhow::Context;

use crate::{Day, FileParser};
use std::collections::VecDeque;
use std::io::Read;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &mut dyn Read) -> anyhow::Result<i64> {
    day1_impl(input, 1)
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<i64> {
    day1_impl(input, 3)
}

fn day1_impl(input: &mut dyn Read, window_size: usize) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);

    let mut increases = 0;
    let mut window_sum: i32 = 0;
    let mut buffer = VecDeque::new();

    for _ in 0..window_size {
        let reading = parser
            .parse_line()
            .context("need at least window_size readings")?;
        window_sum += reading;
        buffer.push_back(reading);
    }

    for new_reading in parser.iter_parse::<i32>() {
        let old_reading = buffer.pop_front().expect("window_size must be > 0");
        buffer.push_back(new_reading);
        let new_window = window_sum + new_reading - old_reading;

        if new_window > window_sum {
            increases += 1;
        }

        window_sum = new_window;
    }

    parser.finish()?;
    Ok(increases as i64)
}

crate::test_day!(RUN, "day1", 1527, 1575);
