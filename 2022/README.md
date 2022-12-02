# Advent of Code 2022

## Running

```
advent-of-code 0.1.0
Solutions for Advent of Code puzzles.

USAGE:
    advent-of-code-2022 [FLAGS] [OPTIONS] --day <day>

FLAGS:
    -a, --all
    -e, --example    Run with example input
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --day <day>
    -i, --input <input>      Override input file
    -p, --part <part>         [default: 1]
    -r, --repeat <repeat>    Repeat the computation many times for easier flamegraphing [default: 1]
```

The input file defaults to `inputs/dayX/input.txt` (or the corresponding `example.txt` if
`--example` is given).

## Benchmarking

Run benchmarks with `cargo bench`. A different set of input files can be specified with the
`AOC_INPUT_DIR` environment variable.