# thisisplural

[![GitHub](https://img.shields.io/badge/GitHub-ryo33/thisisplural-222222)](https://github.com/ryo33/thisisplural)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/thisisplural)](https://crates.io/crates/thisisplural)
[![docs.rs](https://img.shields.io/docsrs/thisisplural)](https://docs.rs/thisisplural)

`#[derive(Plural)]` for creating frictionless new types with any collection
type.

## Features

- Automatically implements `From`, `Into`, `FromIterator`, `IntoIterator`, and
  methods like `.len()` or `::with_capacity`.
- Supports any collection that behaves like `Vec` and `HashMap`.

## Example

```rust
// This implements `From`, `Into`, `FromIterator`, `IntoIterator`.
#[derive(Plural)]
struct Numbers(Vec<u32>);

// use `From` trait
let my_favorite_numbers: Numbers = vec![].into();

// methods like `len()` are implemented
assert_eq!(my_favorite_numbers.len(), 0);
assert!(my_favorite_numbers.is_empty());

// `FromIterator` allows this `collect()`
let doubled_numbers: Numbers = my_favorite_numbers.iter().map(|x| x * 2).collect();

// `HashMap` like collections are also supported
#[derive(Plural)]
struct FavoriteNumbers(HashMap<&'static str, Numbers>);

// construct the struct with using `FromIterator`
let favorite_numbers =
    FavoriteNumbers::from_iter([("ryo33", my_favorite_numbers), ("someone", doubled_numbers)]);

// use it in a `for` loop (`IntoIterator` trait)
for (name, numbers) in favorite_numbers {
    // use Deref trait
    println!("{} has {} favorite number(s)", name, numbers.0.len());
}
```
