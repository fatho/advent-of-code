use anyhow::{bail, Context};

use crate::FileParser;
use std::cmp::Ordering;
use std::io::Read;
use std::str::FromStr;

pub fn part1(input: &mut dyn Read) -> anyhow::Result<()> {
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
        println!("{}", sum * last_draw);
    } else {
        bail!("no win")
    }
    Ok(())
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<()> {
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
        println!("{}", sum * last_draw);
    } else {
        bail!("no win")
    }
    Ok(())
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
    marked: Vec<bool>,
}

impl Board {
    fn new(numbers: Vec<u32>) -> Board {
        Board {
            marked: vec![false; numbers.len()],
            numbers,
        }
    }

    fn won(&self) -> bool {
        for start in 0..BOARD_WIDTH {
            let row = start * BOARD_WIDTH..(start + 1) * BOARD_WIDTH;
            if row.into_iter().all(|i| self.marked[i]) {
                return true;
            }
            let col = (0..BOARD_WIDTH).map(|r| start + r * BOARD_WIDTH);
            if col.into_iter().all(|i| self.marked[i]) {
                return true;
            }
        }
        false
    }

    fn mark(&mut self, number: u32) {
        if let Some(pos) = self.numbers.iter().position(|x| *x == number) {
            self.marked[pos] = true;
        }
    }

    fn sum_unmarked(&self) -> u32 {
        self.numbers
            .iter()
            .zip(self.marked.iter())
            .filter_map(|(n, m)| if *m { None } else { Some(*n) })
            .sum()
    }
}
