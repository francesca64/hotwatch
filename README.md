# hotwatch

[![Cargo Version](http://meritbadge.herokuapp.com/hotwatch)](https://crates.io/crates/hotwatch)
[![Build Status](https://travis-ci.org/francesca64/hotwatch.svg?branch=master)](https://travis-ci.org/francesca64/hotwatch)
[![Clippy Linting Result](https://clippy.bashy.io/github/francesca64/hotwatch/master/badge.svg)](https://clippy.bashy.io/github/francesca64/hotwatch/master/log)

[Documentation](https://francesca64.github.io/hotwatch/docs/hotwatch)

*hotwatch* is a Rust library for conveniently watching and handling file changes.

Nightly Rust is required, since I used the box keyword a few times. Sorry.

```rust
use hotwatch::Hotwatch;

let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
hotwatch.watch("war.png", |e: hotwatch::Event| {
    if e.changed() {
        println!("War has changed.");
    }
}).expect("Failed to watch file!");
```
