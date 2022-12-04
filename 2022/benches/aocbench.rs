use advent_of_code_2022::{self as aoc, include_input_env, Day};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_day(c: &mut Criterion, name: &str, day: Day, input: &[u8]) {
    c.bench_function(format!("{}.1", name).as_ref(), |b| {
        b.iter(|| (day.part1)(black_box(input)).unwrap())
    });
    c.bench_function(format!("{}.2", name).as_ref(), |b| {
        b.iter(|| (day.part2)(black_box(input)).unwrap())
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_day(c, "day1", aoc::day1::RUN, &include_input_env!("day1"));
    bench_day(c, "day2", aoc::day2::RUN, &include_input_env!("day2"));
    bench_day(c, "day3", aoc::day3::RUN, &include_input_env!("day3"));
    bench_day(c, "day4", aoc::day4::RUN, &include_input_env!("day4"));
    // bench_day(c, "day5", aoc::day5::RUN, &include_input_env!("day5"));
    // bench_day(c, "day6", aoc::day6::RUN, &include_input_env!("day6"));
    // bench_day(c, "day7", aoc::day7::RUN, &include_input_env!("day7"));
    // bench_day(c, "day8", aoc::day8::RUN, &include_input_env!("day8"));
    // bench_day(c, "day9", aoc::day9::RUN, &include_input_env!("day9"));
    // bench_day(c, "day10", aoc::day10::RUN, &include_input_env!("day10"));
    // bench_day(c, "day11", aoc::day11::RUN, &include_input_env!("day11"));
    // bench_day(c, "day12", aoc::day12::RUN, &include_input_env!("day12"));
    // bench_day(c, "day13", aoc::day13::RUN, &include_input_env!("day13"));
    // bench_day(c, "day14", aoc::day14::RUN, &include_input_env!("day14"));
    // bench_day(c, "day15", aoc::day15::RUN, &include_input_env!("day15"));
    // bench_day(c, "day16", aoc::day16::RUN, &include_input_env!("day16"));
    // bench_day(c, "day17", aoc::day17::RUN, &include_input_env!("day17"));
    // bench_day(c, "day18", aoc::day18::RUN, &include_input_env!("day18"));
    // bench_day(c, "day19", aoc::day19::RUN, &include_input_env!("day19"));
    // bench_day(c, "day20", aoc::day20::RUN, &include_input_env!("day20"));
    // bench_day(c, "day21", aoc::day21::RUN, &include_input_env!("day21"));
    // bench_day(c, "day22", aoc::day22::RUN, &include_input_env!("day22"));
    // bench_day(c, "day23", aoc::day23::RUN, &include_input_env!("day23"));
    // bench_day(c, "day24", aoc::day24::RUN, &include_input_env!("day24"));
    // bench_day(c, "day25", aoc::day25::RUN, &include_input_env!("day25"));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
