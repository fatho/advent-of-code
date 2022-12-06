#![allow(unused)]

use anyhow::bail;
use nom::{
    bytes::complete::{take, take_until, take_while},
    combinator::{flat_map, map_opt, opt},
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
            pair(
                map_opt(take(N), |bytes: &[u8]| {
                    if bytes.iter().all(|ch| (b'a'..=b'z').contains(ch)) {
                        Some(bytes)
                    } else {
                        None
                    }
                }),
                take_while(|ch| (b'a'..=b'z').contains(&ch)),
            ),
            opt(parsers::newline),
        ),
        input,
    )?;

    let mut buf = <&[u8] as TryInto<[u8; N]>>::try_into(init)
        .expect("parser should've only succeeded with four elements here");
    let mut pos = 0usize;

    // With usize counts it's slower, and u32 should be enough for everything practical. Still,
    // better be safe than sorry.
    assert!(N <= u32::MAX as usize);
    let mut counts = [0u32; 256]; // histogram of all the characters in the window
    let mut num_duplicates = 0; // number of characters that occur more than once in the window

    for b in buf.iter() {
        counts[*b as usize] += 1;
        // count each character only the first time a duplicate is introduced
        if counts[*b as usize] == 2 {
            num_duplicates += 1;
        }
    }

    while num_duplicates > 0 && pos < rest.len() {
        buf.rotate_left(1);
        let new = rest[pos];
        let old = std::mem::replace(&mut buf[3], new);
        // If the chracters in the window changed we need to update our state
        if new != old {
            counts[new as usize] += 1;
            // Check if `new` became a duplicate
            if counts[new as usize] == 2 {
                num_duplicates += 1;
            }
            counts[old as usize] -= 1;
            // Check if `old` no longer is a duplicate
            if counts[old as usize] == 1 {
                num_duplicates -= 1;
            }
        }
        pos += 1;
    }

    Ok((pos + N).to_string())
}

#[test]
fn test_examples() {
    assert_eq!(
        part1(b"mjqjpqmgbljsphdztnvjfqwrcgsmlb").unwrap(),
        "7".to_string()
    );
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
