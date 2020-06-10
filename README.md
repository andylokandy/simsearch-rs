# `simsearch`

[![Build Status](https://travis-ci.com/andylokandy/simsearch-rs.svg?branch=master)](https://travis-ci.com/andylokandy/simsearch-rs)
[![crates.io](https://img.shields.io/crates/v/simsearch.svg)](https://crates.io/crates/simsearch)
[![docs.rs](https://docs.rs/simsearch/badge.svg)](https://docs.rs/simsearch)

A simple and lightweight fuzzy search engine that works in memory, searching for similar strings (a pun here).

### [**Documentation**](https://docs.rs/simsearch)

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
simsearch = "0.2"
```

## Example

```rust
use simsearch::SimSearch;

let mut engine: SimSearch<u32> = SimSearch::new();

engine.insert(1, "Things Fall Apart");
engine.insert(2, "The Old Man and the Sea");
engine.insert(3, "James Joyce");

let results: Vec<u32> = engine.search("thngs");

assert_eq!(results, &[1]);
```
By default, Jaro-Winkler distance is used. SIMD-accelerated Levenshtein distance
for ASCII byte strings is also supported by specifying custom `SearchOptions`:
```rust
use simsearch::{SimSearch, SearchOptions};

let options = SearchOptions::new().levenshtein(true);
let mut engine: SimSearch<u32> = SimSearch::new_with(options);
```

Also try the interactive demo by:

```
$ cargo run --release --example books
```

## Contribution

All kinds of contribution are welcomed.

- **Issus.** Feel free to open an issue when you find typos, bugs, or have any question.
- **Pull requests**. New collection, better implementation, more tests, more documents and typo fixes are all welcomed.

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
