#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while};
use nom::combinator::{all_consuming, flat_map, map, map_res};
use nom::multi::{fold_many0, fold_many1, many0, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let bin_input = hex_to_bin(input);
    let pack = parsers::parse(terminated(p_packet, many0(tag("0"))), &bin_input)?;
    Ok(version_sum(&pack).to_string())
}

fn version_sum(packet: &Packet) -> u32 {
    packet.version as u32
        + match packet.body {
            Body::Literal(_) => 0,
            Body::Operator { ref children, .. } => children.iter().map(version_sum).sum(),
        }
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let bin_input = hex_to_bin(input);
    let pack = parsers::parse(terminated(p_packet, many0(tag("0"))), &bin_input)?;
    Ok(eval(&pack).to_string())
}

fn eval(packet: &Packet) -> u64 {
    match packet.body {
        Body::Literal(val) => val,
        Body::Operator { typ, ref children } => match typ {
            0 => children.iter().map(eval).sum(),
            1 => children.iter().map(eval).product(),
            2 => children.iter().map(eval).min().expect("at least one child"),
            3 => children.iter().map(eval).max().expect("at least one child"),
            5 => (eval(&children[0]) > eval(&children[1])) as u64,
            6 => (eval(&children[0]) < eval(&children[1])) as u64,
            7 => (eval(&children[0]) == eval(&children[1])) as u64,
            other => unimplemented!("unknown type {}", other),
        },
    }
}

// Parser

#[derive(Debug, Clone, PartialEq, Eq)]
struct Packet {
    version: u8,
    body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Body {
    Literal(u64),
    Operator { typ: u8, children: Vec<Packet> },
}

fn p_packet(input: &[u8]) -> IResult<&[u8], Packet> {
    flat_map(
        pair(
            // VERSION
            map(take(3u32), bin_to_int),
            // TYPE
            map(take(3u32), bin_to_int),
        ),
        |(version, typ)| {
            move |input| {
                if typ == 4 {
                    map(p_literal, move |lit| Packet {
                        version: version as u8,
                        body: Body::Literal(lit),
                    })(input)
                } else {
                    map(p_operator, move |ops| Packet {
                        version: version as u8,
                        body: Body::Operator {
                            typ: typ as u8,
                            children: ops,
                        },
                    })(input)
                }
            }
        },
    )(input)
}

fn p_literal(input: &[u8]) -> IResult<&[u8], u64> {
    map(
        pair(
            fold_many0(
                preceded(tag("1"), take(4u32)),
                || 0,
                |acc, block| (acc << 4) + bin_to_int(block),
            ),
            preceded(tag("0"), take(4u32)),
        ),
        |(initial, last_block)| (initial << 4) + bin_to_int(last_block),
    )(input)
}

fn p_operator(input: &[u8]) -> IResult<&[u8], Vec<Packet>> {
    alt((
        // Length type 0
        preceded(
            tag("0"),
            flat_map(take(15u32), |lenstr| {
                let len = bin_to_int(lenstr);
                map_res(take(len), |subinput| {
                    many0(p_packet)(subinput).map(|(_, ret)| ret)
                })
            }),
        ),
        // Length type 1
        preceded(
            tag("1"),
            flat_map(take(11u32), |lenstr| {
                let len = bin_to_int(lenstr) as usize;
                many_m_n(len, len, p_packet)
            }),
        ),
    ))(input)
}

fn bin_to_int(input: &[u8]) -> u64 {
    input.iter().fold(0, |int, b| int * 2 + (*b == b'1') as u64)
}

// Preparation

const HEX_TO_BIN: [&[u8]; 16] = [
    b"0000", // 0
    b"0001", // 1
    b"0010", // 2
    b"0011", // 3
    b"0100", // 4
    b"0101", // 5
    b"0110", // 6
    b"0111", // 7
    b"1000", // 8
    b"1001", // 9
    b"1010", // A
    b"1011", // B
    b"1100", // C
    b"1101", // D
    b"1110", // E
    b"1111", // F
];

// Step 1:
fn hex_to_bin(hex_str: &[u8]) -> Vec<u8> {
    hex_str
        .iter()
        .filter_map(|b| match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .flat_map(|i| HEX_TO_BIN[i as usize])
        .copied()
        .collect()
}

#[test]
fn test_parse_literal() {
    let input = hex_to_bin(b"D2FE28");
    assert_eq!(
        p_packet(&input),
        Ok((
            b"000".as_ref(),
            Packet {
                version: 6,
                body: Body::Literal(2021)
            }
        ))
    );
}

#[test]
fn test_part2_examples() {
    fn check(input: &[u8], result: u64) {
        match part2(input) {
            Ok(ret) => assert_eq!(ret, result.to_string()),
            Err(_) => panic!("could not parse"),
        }
    }
    check(b"C200B40A82", 3);
    check(b"04005AC33890", 54);
    check(b"880086C3E88112", 7);
    check(b"CE00C43D881120", 9);
    check(b"D8005AC2A8F0", 1);
    check(b"F600BC2D8F", 0);
    check(b"9C005AC2F8F0", 0);
    check(b"9C0141080250320F1802104A08", 1);
}

crate::test_day!(crate::day16::RUN, "day16", "1014", "1922490999789");
