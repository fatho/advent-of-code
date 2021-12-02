use crate::BufReadExt;
use std::collections::VecDeque;
use std::io::{self, Read};

pub fn part1(input: &mut dyn Read) -> std::io::Result<()> {
    day1_impl(input, 1)
}

pub fn part2(input: &mut dyn Read) -> std::io::Result<()> {
    day1_impl(input, 3)
}

fn day1_impl(input: &mut dyn Read, window_size: usize) -> std::io::Result<()> {
    let mut reader = io::BufReader::new(input);
    let mut line = String::new();
    let mut increases = 0;
    let mut window_sum: i32 = 0;
    let mut buffer = VecDeque::new();

    for _ in 0..window_size {
        let reading = reader.read_parse::<i32>(&mut line)?;
        window_sum += reading;
        buffer.push_back(reading);
    }

    while let Some(new_reading) = reader.read_parse_or_eof(&mut line)? {
        let old_reading = buffer.pop_front().expect("window_size must be > 0");
        buffer.push_back(new_reading);
        let new_window = window_sum + new_reading - old_reading;

        if new_window > window_sum {
            increases += 1;
        }

        window_sum = new_window;
        line.clear();
    }
    println!("{}", increases);
    Ok(())
}
