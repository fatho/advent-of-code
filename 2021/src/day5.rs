use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let lines = parsers::parse(many0(terminated(Line::parse, parsers::newline)), input)?;

    let (max_x, max_y) = lines
        .iter()
        .flat_map(|l| [l.p1, l.p2])
        .fold((0, 0), |(mx, my), p| (mx.max(p.x), my.max(p.y)));

    let mut map = Detector::new(max_x as u32 + 1, max_y as u32 + 1);

    for line in lines {
        if line.is_axis_aligned() {
            line.points().for_each(|p| map.add_point(p));
        }
    }

    Ok(map.count_danger() as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let lines = parsers::parse(many0(terminated(Line::parse, parsers::newline)), input)?;

    let (max_x, max_y) = lines
        .iter()
        .flat_map(|l| [l.p1, l.p2])
        .fold((0, 0), |(mx, my), p| (mx.max(p.x), my.max(p.y)));

    let mut map = Detector::new(max_x as u32 + 1, max_y as u32 + 1);

    for line in lines {
        line.points().for_each(|p| map.add_point(p));
    }

    Ok(map.count_danger() as i64)
}

struct Detector {
    bits: Vec<u64>,
    stride: u32,
}

impl Detector {
    pub fn new(width: u32, height: u32) -> Self {
        let num_bits = width * height * 2;
        let num_words = (num_bits + 63) / 64;
        Detector {
            bits: vec![0; num_words as usize],
            stride: width,
        }
    }

    fn split_index(&self, x: u32, y: u32) -> (usize, u32) {
        let bit_total = (x * self.stride + y) * 2;
        let word = bit_total >> 6;
        let bit_in_word = bit_total & 0b11_1111;
        (word as usize, bit_in_word)
    }

    pub fn add_point(&mut self, p: Point) {
        let (word_index, bit_index) = self.split_index(p.x as u32, p.y as u32);
        let word = self.bits[word_index];
        let value = (word >> bit_index) & 0b11;
        if value < 2 {
            let new_value = value + 1;
            let mask = 0b11 << bit_index;
            self.bits[word_index] = (word & !mask) | (new_value << bit_index);
        }
    }

    pub fn count_danger(&self) -> u32 {
        let danger_mask = 0xAAAA_AAAA_AAAA_AAAA_u64;
        self.bits
            .iter()
            .map(|word| (word & danger_mask).count_ones())
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(parsers::i32, tag(","), parsers::i32),
            |(x, y)| Self { x, y },
        )(input)
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

struct Line {
    p1: Point,
    p2: Point,
}

impl Line {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(Point::parse, tag(" -> "), Point::parse),
            |(p1, p2)| Self { p1, p2 },
        )(input)
    }

    pub fn is_axis_aligned(&self) -> bool {
        self.p1.x == self.p2.x || self.p1.y == self.p2.y
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        let dx = self.p2.x - self.p1.x;
        let dy = self.p2.y - self.p1.y;
        assert!(
            dx == 0 || dy == 0 || dx.abs() == dy.abs(),
            "lines can only be horizontal, vertical or diagonal"
        );
        let steps = dx.abs().max(dy.abs());
        let stepx = dx.signum();
        let stepy = dy.signum();

        std::iter::successors(Some(self.p1), move |p| Some(p.offset(stepx, stepy)))
            .take(steps as usize + 1)
    }
}

crate::test_day!(crate::day5::RUN, "day5", 7644, 18627);
