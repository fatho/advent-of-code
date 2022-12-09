#![allow(unused)]

use std::collections::HashSet;

use anyhow::bail;
use nom::{
    bytes::complete::{tag, take},
    combinator::{map, map_opt, map_res},
    multi::fold_many0,
    sequence::{separated_pair, terminated},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut rope = Rope::new();
    let mut visited = HashSet::new(); // TODO: hashsets are slow, optimize
    visited.insert(Pos { x: 0, y: 0 });
    parsers::parse(
        fold_many0(
            terminated(parse_move, parsers::newline),
            || (),
            |(), mov| {
                for _ in 0..mov.steps {
                    rope.move_head(mov.dir);
                    visited.insert(rope.tail);
                }
            },
        ),
        input,
    )?;

    Ok(visited.len().to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut rope = LongRope::<10>::new();
    let mut visited = HashSet::new(); // TODO: hashsets are slow, optimize
    visited.insert(Pos { x: 0, y: 0 });
    parsers::parse(
        fold_many0(
            terminated(parse_move, parsers::newline),
            || (),
            |(), mov| {
                for _ in 0..mov.steps {
                    rope.move_head(mov.dir);
                    visited.insert(rope.knots[9]);
                }
            },
        ),
        input,
    )?;

    Ok(visited.len().to_string())
}

fn parse_move(input: &[u8]) -> IResult<&[u8], Move> {
    map(
        separated_pair(
            map_opt(take(1usize), |dir: &[u8]| match dir {
                b"L" => Some(Dir::Left),
                b"R" => Some(Dir::Right),
                b"U" => Some(Dir::Up),
                b"D" => Some(Dir::Down),
                _ => None,
            }),
            tag(" "),
            parsers::u32,
        ),
        |(dir, steps)| Move { dir, steps },
    )(input)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn add_dir(self, dir: Dir, amount: i32) -> Pos {
        match dir {
            Dir::Up => Pos {
                y: self.y - amount,
                ..self
            },
            Dir::Down => Pos {
                y: self.y + amount,
                ..self
            },
            Dir::Left => Pos {
                x: self.x - amount,
                ..self
            },
            Dir::Right => Pos {
                x: self.x + amount,
                ..self
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rope {
    head: Pos,
    tail: Pos,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

struct Move {
    dir: Dir,
    steps: u32,
}

impl Rope {
    fn new() -> Self {
        Rope {
            head: Pos { x: 0, y: 0 },
            tail: Pos { x: 0, y: 0 },
        }
    }

    fn touching(&self) -> bool {
        (-1..=1).contains(&(self.head.x - self.tail.x))
            && (-1..=1).contains(&(self.head.y - self.tail.y))
    }

    fn move_head(&mut self, dir: Dir) {
        // adjust tail after each step
        self.head = self.head.add_dir(dir, 1);

        if !self.touching() {
            let dx = self.head.x - self.tail.x;
            let dy = self.head.y - self.tail.y;

            self.tail = Pos {
                x: self.tail.x + dx.signum(),
                y: self.tail.y + dy.signum(),
            }
        }
    }
}

struct LongRope<const N: usize> {
    knots: [Pos; N],
}

impl<const N: usize> LongRope<N> {
    fn new() -> Self {
        LongRope {
            knots: [Pos { x: 0, y: 0 }; N],
        }
    }

    fn move_head(&mut self, dir: Dir) {
        self.knots[0] = self.knots[0].add_dir(dir, 1);

        for i in 0..N - 1 {
            let dx = self.knots[i].x - self.knots[i + 1].x;
            let dy = self.knots[i].y - self.knots[i + 1].y;

            if !(-1..=1).contains(&dx) || !(-1..=1).contains(&dy) {
                // not touching, adjust position
                self.knots[i + 1] = Pos {
                    x: self.knots[i + 1].x + dx.signum(),
                    y: self.knots[i + 1].y + dy.signum(),
                }
            }
        }
    }
}

#[test]
fn test_large_example() {
    assert_eq!(
        part2(
            br"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20
"
        )
        .unwrap(),
        "36".to_string()
    )
}

crate::test_day!(RUN, "day9", "5878", "2405");
