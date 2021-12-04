//! Various nom parsers that are often useful

use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    IResult,
};

macro_rules! parse_int {
    ($name:ident) => {
        pub fn $name(input: &[u8]) -> IResult<&[u8], $name> {
            map_res(digit1, |num_bytes| {
                let num_str =
                    std::str::from_utf8(num_bytes).expect("digits should always be valid UTF8");
                <$name>::from_str_radix(num_str, 10)
            })(input)
        }
    };
}

parse_int!(i32);
parse_int!(i64);
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
            "parser did not consume whole input, remaining:\n{}",
            String::from_utf8_lossy(rest)
        ))
    }
}

pub fn newline(input: &[u8]) -> IResult<&[u8], ()> {
    map(tag(b"\n"), |_| ())(input)
}
