#![allow(unused)]

use anyhow::bail;
use nom::{
    bytes::complete::{take, take_until, take_while},
    combinator::{flat_map, opt},
    multi::fold_many0,
    sequence::{pair, terminated},
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    shared::<4>(input)
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    shared::<14>(input)
}

fn shared<const N: usize>(input: &[u8]) -> anyhow::Result<String> {
    let (init, rest) = parsers::parse(
        terminated(
            pair(take(N), take_while(|ch| (b'a'..=b'z').contains(&ch))),
            opt(parsers::newline),
        ),
        input,
    )?;

    let mut buf = <&[u8] as TryInto<[u8; N]>>::try_into(init)
        .expect("parser should've only succeeded with four elements here");
    let mut pos = 0usize;

    while !all_distinct(buf) && pos < rest.len() {
        buf.rotate_left(1);
        buf[3] = rest[pos];
        pos += 1;
    }

    Ok((pos + N).to_string())
}

fn all_distinct<const N: usize>(mut buf: [u8; N]) -> bool {
    // For the cases we're working with, a naive O(n^2) solution actually outperforms the O(n
    // log(n)) sort + subsequent O(n) check.
    for i in 0..N - 1 {
        for j in i + 1..N {
            if buf[i] == buf[j] {
                return false;
            }
        }
    }
    true
}

#[test]
fn test_examples() {
    assert_eq!(
        part1(b"bvwbjplbgvbhsrlpgdmjqwftvncz").unwrap(),
        "5".to_string()
    );
    assert_eq!(
        part1(b"nppdvjthqldpwncqszvftbrmjlhg").unwrap(),
        "6".to_string()
    );
    assert_eq!(
        part1(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg").unwrap(),
        "10".to_string()
    );
    assert_eq!(
        part1(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw").unwrap(),
        "11".to_string()
    );
}

crate::test_day!(RUN, "day6", "1625", "2250");
