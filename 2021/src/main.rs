use advent_of_code_2021::{aoc_main, Day, day1, day2};

fn main() -> anyhow::Result<()> {
    aoc_main(&[
        Day { first: day1::part1, second: day1::part2 },
        Day { first: day2::part1, second: day2::part2 },
    ])
}
