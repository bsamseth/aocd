# Advent of Code Data

[![crates.io](https://img.shields.io/crates/v/aocd)](https://crates.io/crates/aocd)

Programaticly get your puzzle input and submit answers, in Rust.

Might be useful for lazy Rustaceans and speed hackers.

Yes, this is [wimglenn's `aocd` Python-package](https://github.com/wimglenn/advent-of-code-data), but for Rust. And
yes, this too tries to cache everything it gets from Advent of Code to spare their servers.

## Example

**Spoiler**: This example does in fact solve one of the AoC puzzles.

```rust
use aocd::*;

#[aocd(2022, 1)]
fn main() {
    let mut elves: Vec<_> = input!()
        .split("\n\n")
        .map(|e| e.lines().map(|l| l.parse::<u32>().unwrap()).sum())
        .collect();
    elves.sort();

    submit!(1, elves.last().unwrap());
    submit!(2, elves.iter().rev().take(3).sum::<u32>());
}
```

## Quickstart

You need to provide your AoC session token in order for this crate to get your personal puzzle input and to be able to
submit answers for you. This is a cookie which is set when you login to AoC. You can find it with your browser
inspector. See [this issue](https://github.com/wimglenn/advent-of-code/issues/1) for a how-to. You can provide it to
`aocd` use any of the following alternatives:

```bash
# Alt 1 (this way doesn't require any environment variables to be set):
mkdir -p ~/.config/aocd
echo "your session cookie here" > ~/.config/aocd/token

# Alt 2:
export AOC_SESSION="or here"

# Alt 3:
export AOC_TOKEN="or here"

# Alt 4:
echo "or here" > some_file
export AOC_TOKEN_PATH=some_file
```

Next, add the crate to your dependencies:
```bash
cargo add aocd
```

In your code, annotate your main function with `#[aocd(year, day)]`, and then use the macros `input!()` and
`submit!(part, answer)` to get your puzzle input and submit anawers, respectively. See the example above.

