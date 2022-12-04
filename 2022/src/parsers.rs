//! Various nom parsers that are often useful

use std::ops::RangeInclusive;

use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{fail, map},
    IResult,
};

macro_rules! parse_int {
    ($name:ident) => {
        pub fn $name(input: &[u8]) -> IResult<&[u8], $name> {
            map(digit1, |digits: &[u8]| {
                digits
                    .iter()
                    .map(|d| d - b'0')
                    .fold(0, |acc, d| acc * 10 + d as $name)
            })(input)
        }
    };
}

parse_int!(i32);
parse_int!(i64);

parse_int!(u8);
parse_int!(u32);
parse_int!(u64);

pub fn parse<'a, O>(
    mut parser: impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, nom::error::Error<&[u8]>>,
    input: &'a [u8],
) -> anyhow::Result<O> {
    let (rest, output) = parser(input).map_err(|err| match err {
        nom::Err::Incomplete(needed) => anyhow::anyhow!("needs more input: {:?}", needed),
        nom::Err::Error(inner) => anyhow::anyhow!(
            "error ({:?}): {}",
            inner.code,
            String::from_utf8_lossy(inner.input)
        ),
        nom::Err::Failure(inner) => anyhow::anyhow!(
            "failure ({:?}): {}",
            inner.code,
            String::from_utf8_lossy(inner.input)
        ),
    })?;
    if rest.is_empty() {
        Ok(output)
    } else {
        Err(anyhow::anyhow!(
            "parser did not consume whole input, remaining:\n{:?}",
            String::from_utf8_lossy(rest)
        ))
    }
}

pub fn asciichar(input: &[u8]) -> IResult<&[u8], char> {
    match input {
        [x, rest @ ..] if *x < 128 => Ok((rest, *x as char)),
        _ => fail(input),
    }
}

pub fn byte(input: &[u8]) -> IResult<&[u8], u8> {
    match input {
        [x, rest @ ..] => Ok((rest, *x)),
        _ => fail(input),
    }
}

pub fn byte_range(valid: RangeInclusive<u8>) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
    move |input| match input {
        [x, rest @ ..] if valid.contains(x) => Ok((rest, *x)),
        _ => fail(input),
    }
}

pub fn newline(input: &[u8]) -> IResult<&[u8], ()> {
    map(tag(b"\n"), |_| ())(input)
}

pub fn newline_or_eof(input: &[u8]) -> IResult<&[u8], ()> {
    match input {
        [b'\n', rest @ ..] => Ok((rest, ())),
        [] => Ok((b"", ())),
        _ => fail(input),
    }
}

#[test]
fn test_asciichar() {
    assert_eq!(asciichar(b"Foo"), Ok((b"oo".as_slice(), 'F')));
    assert!(asciichar(b"\xC4oo").is_err());
    assert!(asciichar(b"").is_err());
}
