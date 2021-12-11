#![allow(unused_imports)]

use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let mut map = parsers::parse(p_map, input)?;

    let mut flashes = 0;
    let mut flash_stack = Vec::new();
    for _ in 0..100 {
        // 1. Increase energy by one
        for (pos, energy) in map.positions_mut() {
            *energy += 1;
            if *energy > 9 {
                // We know that this is the first time that the energy went
                // above 9 this step, because all octopi end a step with <= 9.
                flash_stack.push(pos);
            }
        }
        // 2. Flashing
        while let Some((fx, fy)) = flash_stack.pop() {
            flashes += 1;
            for pos in map.neighbours(fx, fy) {
                let value = &mut map[pos];
                *value += 1;
                // only flash once (for the first increase above 9) in a step
                if *value == 10 {
                    flash_stack.push(pos);
                }
            }
        }
        // 3. Reset
        for energy in map.data.iter_mut() {
            if *energy > 9 {
                *energy = 0;
            }
        }
    }

    Ok(flashes)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let mut map = parsers::parse(p_map, input)?;

    let mut flash_stack = Vec::new();
    let mut step = 0;
    loop {
        step += 1;
        // 1. Increase energy by one
        for (pos, energy) in map.positions_mut() {
            *energy += 1;
            if *energy > 9 {
                // We know that this is the first time that the energy went
                // above 9 this step, because all octopi end a step with <= 9.
                flash_stack.push(pos);
            }
        }
        // 2. Flashing
        while let Some((fx, fy)) = flash_stack.pop() {
            for pos in map.neighbours(fx, fy) {
                let value = &mut map[pos];
                *value += 1;
                // only flash once (for the first increase above 9) in a step
                if *value == 10 {
                    flash_stack.push(pos);
                }
            }
        }
        // 3. Reset
        let mut flashes = 0;
        for energy in map.data.iter_mut() {
            if *energy > 9 {
                *energy = 0;
                flashes += 1;
            }
        }

        // check if synchronized
        if flashes == map.width * map.height {
            break;
        }
    }

    Ok(step)
}

fn p_map(input: &[u8]) -> IResult<&[u8], Map<u8>> {
    flat_map(
        terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
        |first_line| {
            let width = first_line.len();
            fold_many0(
                map_opt(
                    terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
                    move |line| {
                        if line.len() == width {
                            Some(line)
                        } else {
                            None
                        }
                    },
                ),
                move || Map {
                    data: first_line.iter().map(|c| c - b'0').collect(),
                    width: width as u32,
                    height: 1,
                },
                |mut map, line| {
                    map.height += 1;
                    map.data.extend(line.iter().map(|c| c - b'0'));
                    map
                },
            )
        },
    )(input)
}

#[derive(Debug)]
struct Map<T> {
    // TODO: try z-order curve layout rather than row major for better cache
    // locality
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Map<T>
where
    T: Copy,
{
    pub fn new(width: u32, height: u32, default: T) -> Self {
        Self {
            data: vec![default; width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> T {
        self.data[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        self.data[(y * self.width + x) as usize] = value;
    }

    pub fn positions_mut(&mut self) -> impl Iterator<Item = ((u32, u32), &mut T)> + '_ {
        // TODO: eliminate division and mod (expensive!)
        let width = self.width;
        self.data
            .iter_mut()
            .enumerate()
            .map(move |(pos, value)| ((pos as u32 % width, pos as u32 / width), value))
    }

    pub fn neighbours_with_index(
        &self,
        x: u32,
        y: u32,
    ) -> impl Iterator<Item = ((u32, u32), T)> + '_ {
        // TODO: find something nicer than relying on wrapping?
        let positions = [
            (x.wrapping_sub(1), y.wrapping_sub(1)),
            (x, y.wrapping_sub(1)),
            (x + 1, y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x.wrapping_sub(1), y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ];
        positions
            .into_iter()
            .filter(|(nx, ny)| (0..self.width).contains(nx) && (0..self.height).contains(ny))
            .map(|(nx, ny)| ((nx, ny), self.get(nx, ny)))
    }

    pub fn neighbours(&self, x: u32, y: u32) -> impl Iterator<Item = (u32, u32)> + 'static {
        // TODO: find something nicer than relying on wrapping?
        let positions = [
            (x.wrapping_sub(1), y.wrapping_sub(1)),
            (x, y.wrapping_sub(1)),
            (x + 1, y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x.wrapping_sub(1), y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ];
        let width = self.width;
        let height = self.height;
        positions
            .into_iter()
            .filter(move |(nx, ny)| (0..width).contains(nx) && (0..height).contains(ny))
    }
}

impl<T> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1 * self.width) as usize]
    }
}

impl<T> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1 * self.width) as usize]
    }
}

crate::test_day!(crate::day11::RUN, "day11", 0, 0);
