use advent_of_code_2021::{self as aoc, include_input, Day};

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
    bench_day(c, "day1", aoc::day1::RUN, include_input!("day1"));
    bench_day(c, "day2", aoc::day2::RUN, include_input!("day2"));
    bench_day(c, "day3", aoc::day3::RUN, include_input!("day3"));
    bench_day(c, "day4", aoc::day4::RUN, include_input!("day4"));
    bench_day(c, "day5", aoc::day5::RUN, include_input!("day5"));
    bench_day(c, "day6", aoc::day6::RUN, include_input!("day6"));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
