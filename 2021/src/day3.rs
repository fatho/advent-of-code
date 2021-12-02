use crate::FileParser;
use std::io::Read;
use std::str::FromStr;

pub fn part1(input: &mut dyn Read) -> anyhow::Result<()> {
    let mut parser = FileParser::new(input);

    todo!("implement solution here");

    parser.finish()?;

    println!("{}", 0);
    Ok(())
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<()> {
    let mut parser = FileParser::new(input);

    todo!("implement solution here");

    parser.finish()?;

    println!("{}", 0);
    Ok(())
}
