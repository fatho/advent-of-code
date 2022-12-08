use std::{
    fmt::{Display, Write},
    ops::{Index, IndexMut},
};

use anyhow::anyhow;

use crate::Day;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let map = parse_map(input)?;

    let mut vis = Map::new(map.width, map.height, false);

    scan_visibility(&map, &mut vis, map.height, map.width, |x, y| (x, y));
    scan_visibility(&map, &mut vis, map.width, map.height, |y, x| (x, y));

    let total = vis.data.iter().map(|v| *v as u32).sum::<u32>();
    Ok(total.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let map = parse_map(input)?;

    let mut best = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let origin = map[(x, y)];

            // TODO: think hard and optimize
            let vl = scan_dist(&map, origin, (0..x).rev(), |tx| (tx, y));
            let vr = scan_dist(&map, origin, x + 1..map.width, |tx| (tx, y));
            let vt = scan_dist(&map, origin, (0..y).rev(), |ty| (x, ty));
            let vb = scan_dist(&map, origin, y + 1..map.height, |ty| (x, ty));

            let score = vl * vr * vt * vb;
            if score > best {
                best = score;
            }
        }
    }

    Ok(best.to_string())
}

fn scan_visibility(
    map: &Map<u8>,
    vis: &mut Map<bool>,
    side_len: u32,
    other_side_len: u32,
    mk_index: impl Fn(u32, u32) -> (u32, u32),
) {
    for y in 0..side_len {
        let mut l = 0;
        let mut r = other_side_len - 1;

        let mut maxl = map[mk_index(l, y)];
        let mut maxr = map[mk_index(r, y)];

        vis[mk_index(l, y)] = true;
        vis[mk_index(r, y)] = true;

        // Always advance the smaller side
        while l < r {
            if maxl < maxr {
                l += 1;
                if map[mk_index(l, y)] > maxl {
                    maxl = map[mk_index(l, y)];
                    vis[mk_index(l, y)] = true;
                }
            } else {
                r -= 1;
                if map[mk_index(r, y)] > maxr {
                    maxr = map[mk_index(r, y)];
                    vis[mk_index(r, y)] = true;
                }
            }
        }
    }
}

fn scan_dist(
    map: &Map<u8>,
    origin: u8,
    mut range: impl Iterator<Item = u32>,
    mk_coord: impl Fn(u32) -> (u32, u32),
) -> u32 {
    range
        .try_fold(0, |dist, tx| {
            if map[mk_coord(tx)] < origin {
                Ok(dist + 1)
            } else {
                Err(dist + 1)
            }
        })
        .unwrap_or_else(|x| x)
}

fn parse_map(input: &[u8]) -> anyhow::Result<Map<u8>> {
    let mut width = 0;
    let mut cur_line = 0;
    let mut height = 0;
    let trees = input
        .iter()
        .copied()
        .filter_map(|ch| match ch {
            b'0'..=b'9' => {
                cur_line += 1;
                Some(Ok(ch - b'0'))
            }
            b'\n' => {
                height += 1;
                if width == 0 {
                    width = cur_line;
                }

                if width == cur_line {
                    cur_line = 0;
                    None
                } else {
                    Some(Err(anyhow!("Row width mismatch")))
                }
            }
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?;

    // handle optional last \n
    if cur_line > 0 {
        height += 1
    }

    Ok(Map {
        data: trees,
        width,
        height,
    })
}

struct Map<T> {
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T: Clone> Map<T> {
    fn new(width: u32, height: u32, init: T) -> Self {
        Map {
            data: vec![init; width as usize * height as usize],
            width,
            height,
        }
    }

    fn offset(&self, x: u32, y: u32) -> usize {
        (x as usize) + (self.width as usize) * (y as usize)
    }
}

impl<T: Clone> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[self.offset(index.0, index.1)]
    }
}

impl<T: Clone> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        let offset = self.offset(index.0, index.1);
        &mut self.data[offset]
    }
}

impl Display for Map<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.chunks(self.width as usize) {
            for col in row {
                f.write_char((col + b'0') as char)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[test]
fn test_map() {
    const INPUT: &[u8] = b"30373\n25512\n65332\n33549\n35390\n";
    assert_eq!(
        parse_map(INPUT).unwrap().to_string(),
        String::from_utf8(INPUT.to_owned()).unwrap()
    );
}

crate::test_day!(RUN, "day8", "1779", "172224");
