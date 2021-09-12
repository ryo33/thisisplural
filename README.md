# thisisplural

[![GitHub](https://img.shields.io/badge/GitHub-ryo33/thisisplural-222222)](https://github.com/ryo33/thisisplural)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/thisisplural)](https://crates.io/crates/thisisplural)
[![docs.rs](https://img.shields.io/docsrs/thisisplural)](https://docs.rs/thisisplural)

`#[derive(Plural)]` for creating frictionless new types with a `Vec`, `HashMap`, etc.

## Features

- Automatically implements `From`, `Into`, `FromIterator`, `IntoIterator`, `Deref`, and `DerefMut`.
- Supports `Vec` and `HashMap` (adding other collections to here is very easy).

## Example

```rust
// This implements `From`, `Into`, `FromIterator`, `IntoIterator`, `Deref`, and `DerefMut`.
#[derive(Plural)]
struct Numbers(Vec<u32>);

// use `From` trait
let mut my_favorite_numbers: Numbers = vec![].into();

// use inner APIs with `DerefMut` trait.
my_favorite_numbers.push(42);

// `FromIterator` allows this `collect()`
let doubled_numbers: Numbers = my_favorite_numbers.iter().map(|x| x * 2).collect();

// `HashMap` is also supported
#[derive(Plural)]
struct FavoriteNumbers(HashMap<&'static str, Numbers>);

// construct the struct with using `FromIterator`
let favorite_numbers =
    FavoriteNumbers::from_iter([("ryo33", my_favorite_numbers), ("someone", doubled_numbers)]);

// use it in a `for` loop (`IntoIterator` trait)
for (name, numbers) in favorite_numbers {
    // use Deref trait
    println!("{} has {} favorite number(s)", name, numbers.len());
}
```
