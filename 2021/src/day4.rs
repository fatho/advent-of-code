use anyhow::bail;
use nom::{
    bytes::{complete::take_while, streaming::tag},
    combinator::map,
    multi::{many0, many_m_n, separated_list0},
    sequence::{pair, terminated},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let mut bingo = parsers::parse(p_bingo, input)?;

    let mut win = None;

    'bingo: for draw in bingo.draws {
        for board in bingo.boards.iter_mut() {
            board.mark(draw);
            if board.won() {
                win = Some((board.sum_unmarked(), draw));
                break 'bingo;
            }
        }
    }

    if let Some((sum, last_draw)) = win {
        Ok((sum * last_draw) as i64)
    } else {
        bail!("no win")
    }
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let mut bingo = parsers::parse(p_bingo, input)?;

    let mut last_win = None;

    for draw in bingo.draws {
        for board in bingo.boards.iter_mut() {
            board.mark(draw);
        }
        bingo.boards.retain(|board| {
            if board.won() {
                last_win = Some((board.sum_unmarked(), draw));
                false
            } else {
                true
            }
        });
        if bingo.boards.is_empty() {
            break;
        }
    }

    if let Some((sum, last_draw)) = last_win {
        Ok((sum * last_draw) as i64)
    } else {
        bail!("no win")
    }
}

struct Bingo {
    draws: Vec<u32>,
    boards: Vec<Board>,
}

fn p_board(input: &[u8]) -> IResult<&[u8], Board> {
    const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_WIDTH;
    map(
        many_m_n(
            BOARD_SIZE,
            BOARD_SIZE,
            terminated(parsers::u32, take_while(|x: u8| x.is_ascii_whitespace())),
        ),
        |numbers| Board::new(numbers),
    )(input)
}

fn p_bingo(input: &[u8]) -> IResult<&[u8], Bingo> {
    map(
        pair(
            // first line holds the numbers to be drawn
            terminated(
                separated_list0(tag(","), parsers::u32),
                take_while(|x: u8| x.is_ascii_whitespace()),
            ),
            // followed by many boards
            many0(p_board),
        ),
        |(draws, boards)| Bingo { draws, boards },
    )(input)
}

const BOARD_WIDTH: usize = 5;

#[derive(Debug, Clone)]
struct Board {
    /// row-major representation of the board
    numbers: Vec<u32>,
    marked: u32,
}

impl Board {
    fn new(numbers: Vec<u32>) -> Board {
        // Make sure our board actually fits in a u32
        const _: () = assert!(std::mem::size_of::<u32>() <= BOARD_WIDTH * BOARD_WIDTH);
        Board { numbers, marked: 0 }
    }

    fn won(&self) -> bool {
        const WINNING: &'static [u32] = &[
            // Rows
            0b0000000000000000000011111,
            0b0000000000000001111100000,
            0b0000000000111110000000000,
            0b0000011111000000000000000,
            0b1111100000000000000000000,
            // Columns
            0b0000100001000010000100001,
            0b0001000010000100001000010,
            0b0010000100001000010000100,
            0b0100001000010000100001000,
            0b1000010000100001000010000,
        ];
        for mask in WINNING {
            if self.marked & mask == *mask {
                return true;
            }
        }
        false
    }

    fn mark(&mut self, number: u32) {
        if let Some(pos) = self.numbers.iter().position(|x| *x == number) {
            self.marked |= 1 << pos;
        }
    }

    fn sum_unmarked(&self) -> u32 {
        self.numbers
            .iter()
            .fold((self.marked, 0), |(marked, sum), num| {
                (
                    marked >> 1,
                    sum + (1 - (marked & 1)) * num
                )
            })
            .1
    }
}

crate::test_day!(crate::day4::RUN, "day4", 72770, 13912);
