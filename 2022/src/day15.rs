use std::collections::HashSet;

use anyhow::bail;
use nom::{
    bytes::complete::tag,
    character::complete::i32 as parse_i32,
    combinator::map,
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input)?;

    let result = count_row(&sensors, 2000000)?;
    Ok(result.to_string())
}

fn count_row(sensors: &[Sensor], row: i32) -> anyhow::Result<usize> {
    let mut blocked = Vec::new();
    let mut beacon_xs = HashSet::new();

    for s in sensors.iter() {
        let range = s.beacon_distance();
        let dy = s.position.y.abs_diff(row);
        if dy <= range {
            let rx = range - dy;
            if s.beacon.y == row {
                beacon_xs.insert(s.beacon.x);
            }
            blocked.push((s.position.x - rx as i32, s.position.x + rx as i32 + 1));
        }
    }

    blocked.sort_unstable_by_key(|(from, _)| *from);

    // coalesce overlapping intervals
    let mut count = 0;
    let mut current = blocked[0];
    for next in &blocked[1..] {
        if next.0 > current.1 {
            count += (current.1 - current.0) as usize;
            current = *next;
        } else {
            current.1 = current.1.max(next.1);
        }
    }
    count += (current.1 - current.0) as usize;

    Ok(count - beacon_xs.len())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input)?;
    find_beacon(sensors, 4000000).map(|r| r.to_string())
}

fn find_beacon(mut sensors: Vec<Sensor>, max_coord: i32) -> anyhow::Result<u64> {
    sensors.sort_unstable_by_key(|s| s.position.x);

    let mut target = None;
    let ranges = sensors
        .iter()
        .map(|s| s.beacon_distance())
        .collect::<Vec<_>>();

    for y in 0..=max_coord {
        let mut x = 0;
        for (s, range) in sensors.iter().zip(ranges.iter().copied()) {
            let dy = s.position.y.abs_diff(y);
            if dy <= range {
                let dx = s.position.x.abs_diff(x);
                let rx = range - dy;
                if dx <= rx {
                    x = s.position.x + rx as i32 + 1;
                }
            }
        }
        if x <= max_coord {
            target = Some(Pos { x, y });
            break;
        }
    }

    if let Some(target) = target {
        let tuning = target.x as u64 * 4000000 + target.y as u64;
        Ok(tuning)
    } else {
        bail!("beacon not found");
    }
}

fn parse_sensor(input: &[u8]) -> IResult<&[u8], Sensor> {
    map(
        separated_pair(
            preceded(tag("Sensor at "), parse_pos),
            tag(": "),
            preceded(tag("closest beacon is at "), parse_pos),
        ),
        |(position, beacon)| Sensor { position, beacon },
    )(input)
}

fn parse_pos(input: &[u8]) -> IResult<&[u8], Pos> {
    map(
        separated_pair(
            preceded(tag("x="), parse_i32),
            tag(", "),
            preceded(tag("y="), parse_i32),
        ),
        |(x, y)| Pos { x, y },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn manhattan(self, other: Pos) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sensor {
    position: Pos,
    beacon: Pos,
}

impl Sensor {
    fn beacon_distance(&self) -> u32 {
        self.beacon.manhattan(self.position)
    }
}

#[test]
fn test_example() {
    let input = include_bytes!("../inputs/day15/example.txt");
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input).unwrap();

    let result = count_row(&sensors, 10).unwrap();

    assert_eq!(result, 26);

    let result2 = find_beacon(sensors, 20).unwrap();

    assert_eq!(result2, 56000011)
}

crate::test_day!(RUN, "day15", "5607466", "12543202766584");
