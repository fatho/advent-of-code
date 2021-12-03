#!/usr/bin/env bash

cargo build --release
exe=./target/release/advent-of-code-2021

failed=0

expect() {
    local day=$1
    local part=$2
    local expected=$3
    local actual=$($exe --day $day --part $part --input inputs/day$day/input.txt 2> /dev/null)

    if [[ $expected -eq $actual ]]; then
        echo -e "$day.$part: \u001b[1m\u001b[32mok\u001b[0m"
    else
        echo -e "$day.$part: \u001b[1m\u001b[31mfail\u001b[0m"
        failed=1
    fi
}

check_day() {
    local day=$1
    local expected_1=$2
    local expected_2=$3
    expect $day 1 $expected_1
    expect $day 2 $expected_2
}

end_tests() {
    if [[ $failed -gt 0 ]]; then
        exit 1
    fi
}

check_day 1 1527 1575
check_day 2 1507611 1880593125
check_day 3 4147524 3570354
end_tests