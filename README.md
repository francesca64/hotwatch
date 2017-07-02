# hotwatch

[![Cargo Version](http://meritbadge.herokuapp.com/hotwatch)](https://crates.io/crates/hotwatch)
[![Build Status](https://travis-ci.org/francesca64/hotwatch.svg?branch=master)](https://travis-ci.org/francesca64/hotwatch)

[Documentation](https://francesca64.github.io/hotwatch/docs/hotwatch)

*hotwatch* is a Rust library for comfortably watching and handling file changes. It's a thin convenience wrapper over [notify](https://github.com/passcod/notify), allowing you to easily spawn handlers.

Nightly Rust is required, since I used the box keyword a few times. Sorry.

```rust
use hotwatch::{Hotwatch, Event};

let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
hotwatch.watch("war.png", |e: Event| {
    if let Event::Write(path) = e {
        println!("War has changed.");
    }
}).expect("Failed to watch file!");
```
