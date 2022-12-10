use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::fold_many0,
    sequence::{preceded, terminated},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let probes = [20, 60, 100, 140, 180, 220];

    let (_, _, sum) = parsers::parse(
        fold_many0(
            terminated(parse_instr, parsers::newline),
            || (State::new(), 0, 0),
            |(state, mut next_probe, mut sum), instr| {
                let new_state = state.advance(instr);
                for cyc in state.cycle..new_state.cycle {
                    if next_probe < probes.len() && cyc == probes[next_probe] {
                        next_probe += 1;
                        sum += cyc as i32 * state.x;
                    }
                }
                (new_state, next_probe, sum)
            },
        ),
        input,
    )?;

    Ok(sum.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut crt = Crt::new();

    parsers::parse(
        fold_many0(
            terminated(parse_instr, parsers::newline),
            || (),
            |(), instr| {
                crt.advance(instr);
            },
        ),
        input,
    )?;

    Ok(crt.render())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instr {
    Noop,
    Addx(i32),
}

fn parse_instr(input: &[u8]) -> IResult<&[u8], Instr> {
    alt((
        map(
            preceded(tag("addx "), nom::character::complete::i32),
            Instr::Addx,
        ),
        map(tag("noop"), |_| Instr::Noop),
    ))(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
    cycle: usize,
    x: i32,
}

impl State {
    fn new() -> Self {
        State { cycle: 1, x: 1 }
    }

    fn advance(self, instr: Instr) -> Self {
        match instr {
            Instr::Noop => State {
                cycle: self.cycle + 1,
                x: self.x,
            },
            Instr::Addx(amount) => State {
                cycle: self.cycle + 2,
                x: self.x + amount,
            },
        }
    }
}

struct Crt {
    buf: Vec<bool>,
    cpu: State,
    x: i32,
    y: i32,
}

impl Crt {
    const WIDTH: usize = 40;
    const HEIGHT: usize = 6;

    fn new() -> Crt {
        Crt {
            buf: vec![false; Self::WIDTH * Self::HEIGHT],
            cpu: State::new(),
            x: 0,
            y: 0,
        }
    }

    fn advance(&mut self, instr: Instr) {
        let new_state = self.cpu.advance(instr);
        for _ in self.cpu.cycle..new_state.cycle {
            let dx = self.x - self.cpu.x;

            self.buf[self.x as usize + self.y as usize * Self::WIDTH] = (-1..=1).contains(&dx);

            self.x += 1;
            if self.x as usize == Self::WIDTH {
                self.x = 0;
                self.y += 1;
            }
        }

        self.cpu = new_state;
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for row in self.buf.chunks(Self::WIDTH) {
            for col in row {
                out.push(if *col { '#' } else { '.' });
            }
            out.push('\n');
        }
        out
    }
}

crate::test_day!(
    RUN,
    "day10",
    "13180",
    r"####.####.####..##..#..#...##..##..###..
#.......#.#....#..#.#..#....#.#..#.#..#.
###....#..###..#....####....#.#..#.###..
#.....#...#....#....#..#....#.####.#..#.
#....#....#....#..#.#..#.#..#.#..#.#..#.
####.####.#.....##..#..#..##..#..#.###..
"
);
