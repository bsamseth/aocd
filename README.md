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
