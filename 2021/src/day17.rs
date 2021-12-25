#![allow(unused_imports)]

use std::ops::RangeInclusive;

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map, opt};
use nom::multi::fold_many0;
use nom::sequence::{pair, preceded, separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let target = parsers::parse(terminated(p_target, opt(parsers::newline)), input)?;

    if *target.y.end() >= 0 {
        anyhow::bail!("only works for targets with y < 0");
    }

    let vy_max = target.y.start().abs();

    let mut highest = 0;
    for vx in 1..=*target.x.end() {
        let xsteps = determine_x_steps(0, vx, &target);

        let (min_step, max_step) = match xsteps {
            XResult::Miss => continue,
            XResult::AtLeast(m) => (m, None),
            XResult::Between(m, n) => (m, Some(n)),
        };

        for init_vy in 0..=vy_max {
            let mut step = min_step;
            let mut y = predict_y(0, init_vy, min_step);
            let mut vy = predict_vy(init_vy, min_step);
            loop {
                if target.y.contains(&y) {
                    highest = highest.max(compute_max_y(0, init_vy));
                    break;
                }
                if y < *target.y.start() {
                    break;
                }
                step += 1;
                y += vy;
                vy -= 1;
                if max_step.map_or(false, |max| step > max) {
                    break;
                }
            }
        }
    }

    Ok(highest.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let target = parsers::parse(terminated(p_target, opt(parsers::newline)), input)?;

    if *target.y.end() >= 0 {
        anyhow::bail!("only works for targets with y < 0");
    }
    let vy_max = target.y.start().abs();

    let mut count = 0;
    for vx in 0..=*target.x.end() {
        let xsteps = determine_x_steps(0, vx, &target);

        let (min_step, max_step) = match xsteps {
            XResult::Miss => continue,
            XResult::AtLeast(m) => (m, None),
            XResult::Between(m, n) => (m, Some(n)),
        };

        for init_vy in -vy_max..=vy_max {
            let mut step = min_step;
            let mut y = predict_y(0, init_vy, min_step);
            let mut vy = predict_vy(init_vy, min_step);
            loop {
                if target.y.contains(&y) {
                    count += 1;
                    break;
                }
                if y < *target.y.start() {
                    break;
                }
                step += 1;
                y += vy;
                vy -= 1;
                if max_step.map_or(false, |max| step > max) {
                    break;
                }
            }
        }
    }

    Ok(count.to_string())
}

fn predict_x(start_x: i32, vel_x: i32, steps: u32) -> i32 {
    let offset = if steps as i32 >= vel_x {
        vel_x * (vel_x + 1) / 2
    } else {
        let vel_x_end = vel_x - steps as i32;
        vel_x * (vel_x + 1) / 2 - vel_x_end * (vel_x_end + 1) / 2
    };
    start_x + offset
}

fn predict_y(start_y: i32, vel_y: i32, steps: u32) -> i32 {
    let isteps = steps as i32;
    start_y + isteps * vel_y - isteps * (isteps - 1) / 2
}

fn predict_vy(vel_y: i32, steps: u32) -> i32 {
    let isteps = steps as i32;
    vel_y - isteps
}

fn compute_max_y(start_y: i32, vel_y: i32) -> i32 {
    if vel_y <= 0 {
        start_y
    } else {
        start_y + vel_y * (vel_y + 1) / 2
    }
}

fn determine_x_steps(start_x: i32, vel_x: i32, target: &Target) -> XResult {
    // check if we end in target region
    let final_x = predict_x(start_x, vel_x, vel_x as u32);

    let left_boundary = |s| predict_x(start_x, vel_x, s) >= *target.x.start();
    let right_boundary = |s| predict_x(start_x, vel_x, s) > *target.x.end();

    if final_x < *target.x.start() {
        // undershot
        XResult::Miss
    } else if final_x > *target.x.end() {
        // find when we entered
        let first_beyond_start =
            binary_search(0, vel_x as u32, left_boundary).expect("must have boundary");
        let first_x_beyond_start = predict_x(start_x, vel_x, first_beyond_start);
        if first_x_beyond_start > *target.x.end() {
            // overshot
            XResult::Miss
        } else {
            let first_beyond_end = binary_search(first_beyond_start, vel_x as u32, right_boundary)
                .expect("must have boundary");
            XResult::Between(first_beyond_start, first_beyond_end - 1)
        }
    } else {
        // still inside x-range after velocity runs out

        // find when we entered
        let first_inside =
            binary_search(0, vel_x as u32, left_boundary).expect("must have boundary");
        XResult::AtLeast(first_inside)
    }
}

fn binary_search(mut low: u32, mut high: u32, cond: impl Fn(u32) -> bool) -> Option<u32> {
    use std::cmp::Ordering::*;
    loop {
        match low.cmp(&high) {
            Less => {
                let mid = (low + high) / 2;
                if cond(mid) {
                    high = mid;
                } else {
                    low = mid + 1;
                }
            }
            Equal => {
                return if cond(low) {
                    Some(low)
                } else {
                    // all false
                    None
                };
            }
            Greater => return None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum XResult {
    Miss,
    AtLeast(u32),
    Between(u32, u32),
}

// #[derive(Debug, Clone, Copy)]
// enum YResult {
//     Miss,
//     Between(u32, u32),
// }

fn p_target(input: &[u8]) -> IResult<&[u8], Target> {
    map(
        preceded(
            tag("target area: "),
            separated_pair(
                preceded(
                    tag("x="),
                    separated_pair(numbers::i32, tag(".."), numbers::i32),
                ),
                tag(", "),
                preceded(
                    tag("y="),
                    separated_pair(numbers::i32, tag(".."), numbers::i32),
                ),
            ),
        ),
        |((x1, x2), (y1, y2))| Target {
            x: x1..=x2,
            y: y1..=y2,
        },
    )(input)
}

#[derive(Debug)]
struct Target {
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
}

crate::test_day!(crate::day17::RUN, "day17", "7750", "4120");
