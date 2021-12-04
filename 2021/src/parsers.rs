//! Various nom parsers that are often useful

use nom::{
    bytes::streaming::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    error::ParseError,
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

pub fn parse<'a, O, E: ParseError<&'a [u8]>>(
    mut parser: impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>,
    input: &'a [u8],
) -> anyhow::Result<O> {
    let (rest, output) = parser(input).map_err(|_| anyhow::anyhow!("no parse"))?;
    if rest.is_empty() {
        Ok(output)
    } else {
        Err(anyhow::anyhow!("parser did not consume whole input"))
    }
}

pub fn newline(input: &[u8]) -> IResult<&[u8], ()> {
    map(tag(b"\n"), |_| ())(input)
}
