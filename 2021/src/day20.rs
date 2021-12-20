#![allow(unused_imports)]

use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

// TODO: super hacky

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (algo, map) = parsers::parse(p_input, input)?;
    let mut result = map.pad(10, 10, 10, 10, false);
    println!("{}", render(&result));
    for _ in 0..2 {
        result = convolve(&result, &algo);
        println!("{}", render(&result));
    }
    result = result.shrink(5, 5, 5, 5, false);
    println!("{}", render(&result));
    let num_light = result.data.iter().filter(|b| **b).count();
    Ok(num_light.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (algo, map) = parsers::parse(p_input, input)?;
    let num_enhancements = 50;

    let mut result = map.pad(
        num_enhancements * 2,
        num_enhancements * 2,
        num_enhancements * 2,
        num_enhancements * 2,
        false,
    );
    for _ in 0..num_enhancements {
        result = convolve(&result, &algo);
    }
    result = result.shrink(
        num_enhancements,
        num_enhancements,
        num_enhancements,
        num_enhancements,
        false,
    );
    let num_light = result.data.iter().filter(|b| **b).count();
    Ok(num_light.to_string())
}

fn convolve(map: &Map<bool>, kernel: &[bool]) -> Map<bool> {
    let mut new_map = Map::new(map.width, map.height, false);
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let mut index = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let bit = map[((x as i32 + dx) as u32, (y as i32 + dy) as u32)];
                    index <<= 1;
                    index |= bit as usize;
                }
            }
            new_map[(x, y)] = kernel[index];
        }
    }
    new_map
}

fn render(map: &Map<bool>) -> String {
    let mut out = String::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let ch = if map[(x, y)] { '#' } else { '.' };
            out.push(ch);
        }
        out.push('\n');
    }
    out
}

fn p_input(input: &[u8]) -> IResult<&[u8], (Vec<bool>, Map<bool>)> {
    separated_pair(p_algo, parsers::newline, p_map)(input)
}

fn p_algo(input: &[u8]) -> IResult<&[u8], Vec<bool>> {
    map_opt(
        terminated(take_while(|c| matches!(c, b'.' | b'#')), parsers::newline),
        |algo| {
            if algo.len() == 512 {
                Some(algo.iter().map(|c| *c == b'#').collect())
            } else {
                None
            }
        },
    )(input)
}

fn p_map(input: &[u8]) -> IResult<&[u8], Map<bool>> {
    flat_map(
        terminated(take_while(|c| matches!(c, b'.' | b'#')), parsers::newline),
        |first_line| {
            let width = first_line.len();
            fold_many0(
                map_opt(
                    terminated(take_while(|c| matches!(c, b'.' | b'#')), parsers::newline),
                    move |line| {
                        if line.len() == width {
                            Some(line)
                        } else {
                            None
                        }
                    },
                ),
                move || Map {
                    data: first_line.iter().map(|c| *c == b'#').collect(),
                    width: width as u32,
                    height: 1,
                },
                |mut map, line| {
                    map.height += 1;
                    map.data.extend(line.iter().map(|c| *c == b'#'));
                    map
                },
            )
        },
    )(input)
}

#[derive(Debug)]
struct Map<T> {
    // Presumably no need to bother with Z-order curve here since whole data
    // (100 bytes) fits into two cache lines already (typically 128 bytes).
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Map<T>
where
    T: Copy,
{
    pub fn new(width: u32, height: u32, value: T) -> Self {
        Self {
            data: vec![value; width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn pad(&self, left: u32, right: u32, top: u32, bottom: u32, value: T) -> Map<T> {
        let mut new_map = Map::new(self.width + left + right, self.height + top + bottom, value);
        for y in 0..self.height {
            new_map.data[((y + top) * new_map.width + left) as usize
                ..((y + top) * new_map.width + left + self.width) as usize]
                .copy_from_slice(
                    &self.data[(y * self.width) as usize..(y * self.width + self.width) as usize],
                )
        }
        new_map
    }

    pub fn shrink(&self, left: u32, right: u32, top: u32, bottom: u32, value: T) -> Map<T> {
        let mut new_map = Map::new(self.width - left - right, self.height - top - bottom, value);
        for y in 0..new_map.height {
            new_map.data[(y * new_map.width) as usize..(y * new_map.width + new_map.width) as usize]
                .copy_from_slice(
                    &self.data[((y + top) * self.width + left) as usize
                        ..((y + top) * self.width + left + new_map.width) as usize],
                )
        }
        new_map
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

// TODO: implement truly infinite image:
// struct Image {
//     chunks: HashMap<(i32, i32), u64>,
// }

// impl Image {
//     const CHUNK_EDGE_SIZE: i32 = 8;
//     const CHUNK_EDGE_MASK: i32 = 0b111;

//     /// Split world coordinates into chunk and bit offset
//     fn split_index_flat(&self, x: i32, y: i32) -> ((i32, i32), u8) {
//         let ((chunk_x, chunk_y), (bit_x, bit_y)) = self.split_index(x, y);
//         let bit = (bit_x + bit_y * Self::CHUNK_EDGE_SIZE as u8);
//         ((chunk_x, chunk_y), bit)
//     }

//     fn split_index(&self, x: i32, y: i32) -> ((i32, i32), (u8, u8)) {
//         let chunk_x = x & !Self::CHUNK_EDGE_MASK;
//         let chunk_y = y & !Self::CHUNK_EDGE_MASK;
//         let bit_x = x & Self::CHUNK_EDGE_MASK;
//         let bit_y = y & Self::CHUNK_EDGE_MASK;
//         ((chunk_x, chunk_y), (bit_x as u8, bit_y as u8))
//     }

//     fn kernel3(&self, cx: i32, cy: i32) -> u16 {
//         let ((cc_x, cc_y), (b_x, b_y)) = self.split_index(cx, cy);
//         let center = self.get_chunk(cc_x, cc_y);

//         if b_x == 0 {
//             // Also need things on the left
//             todo!()
//         } else if b_x == 7 {
//             // Also need things on the right
//             todo!()
//         } else {
//             // Only need center column
//             if b_y == 0 {
//                 // Also need top row
//                 todo!()
//             } else if b_y == 7 {
//                 // Also need bottom row
//                 todo!()
//             } else {
//                 // Only need this chunk
//                 (Self::chunk_row::<3>(center, b_x - 1, b_y - 1)
//                     | Self::chunk_row::<3>(center, b_x - 1, b_y - 1) << 3
//                     | Self::chunk_row::<3>(center, b_x - 1, b_y - 1) << 6) as u16
//             }
//         }
//     }

//     fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> u64 {
//         self.chunks.get(&(chunk_x, chunk_y)).unwrap_or(0)
//     }

//     fn chunk_row<const LEN: u8>(chunk: u64, bit_x_start: u8, bit_y: u8) -> u64 {
//         let whole_row = chunk >> (bit_y * Self::CHUNK_EDGE_SIZE as u8);
//         (whole_row >> bit_x_start) & ((1 << LEN) - 1)
//     }
// }

// #[cfg(test)]
// mod chunk_test {
//     use super::*;

//     #[test]
//     fn split_index() {
//         let img = Image {
//             chunks: Default::default(),
//         };
//         assert_eq!(img.split_index_flat(7, 9), ((0, 8), 15));
//         assert_eq!(img.split_index_flat(-7, -9), ((-8, -16), 57));
//     }

//     #[test]
//     fn chunk_row() {
//         let chunk = 0b1111_1111_0110_1001_0001_1000;
//         assert_eq!(Image::chunk_row::<3>(chunk, 1, 1), 0b100);
//         assert_eq!(Image::chunk_row::<3>(chunk, 5, 1), 0b011);
//     }
// }

//crate::test_day!(crate::day20::RUN, "day20", "not solved", "not solved");
