use anyhow::{bail, Context};

use crate::{Day, FileParser};
use std::io::Read;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);

    let draws: Vec<u32> = parser
        .read_line()
        .context("bingo draws")?
        .split(',')
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .context("parsing draws")?;

    if !matches!(parser.read_line(), Some("")) {
        bail!("missing empty line")
    }

    let mut boards = Vec::new();
    while let Some(board) = parse_board(&mut parser) {
        boards.push(board)
    }

    parser.finish()?;
    let mut win = None;

    'bingo: for draw in draws {
        for board in boards.iter_mut() {
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

pub fn part2(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);

    let draws: Vec<u32> = parser
        .read_line()
        .context("bingo draws")?
        .split(',')
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .context("parsing draws")?;

    if !matches!(parser.read_line(), Some("")) {
        bail!("missing empty line")
    }

    let mut boards = Vec::new();
    while let Some(board) = parse_board(&mut parser) {
        boards.push(board)
    }

    parser.finish()?;

    let mut last_win = None;
    let mut board_won = vec![false; boards.len()];

    for draw in draws {
        for (i, board) in boards.iter_mut().enumerate() {
            board.mark(draw);
            if !board_won[i] && board.won() {
                board_won[i] = true;
                last_win = Some((board.sum_unmarked(), draw));
            }
        }
    }

    if let Some((sum, last_draw)) = last_win {
        Ok((sum * last_draw) as i64)
    } else {
        bail!("no win")
    }
}

fn parse_board<R: Read>(parser: &mut FileParser<R>) -> Option<Board> {
    let mut numbers = Vec::new();
    for _ in 0..BOARD_WIDTH {
        for num in parser
            .read_line()?
            .split_whitespace()
            .map(|s| s.parse::<u32>())
        {
            numbers.push(num.ok()?)
        }
    }
    if matches!(parser.read_line(), Some("") | None) {
        Some(Board::new(numbers))
    } else {
        None
    }
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
            if self.marked & mask == *mask { return true; }
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
            .enumerate()
            .filter_map(|(index, num)| {
                if self.marked & (1 << index) == 0 {
                    Some(*num)
                } else {
                    None
                }
            })
            .sum()
    }
}

crate::test_day!(crate::day4::RUN, "day4", 72770, 13912);
