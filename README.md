# hotwatch

[![Crates.io](https://img.shields.io/crates/v/hotwatch.svg)](https://crates.io/crates/hotwatch)
[![Docs.rs](https://docs.rs/hotwatch/badge.svg)](https://docs.rs/hotwatch)
[![CI Status](https://github.com/francesca64/hotwatch/workflows/CI/badge.svg)](https://github.com/francesca64/hotwatch/actions)

`hotwatch` is a Rust library for comfortably watching and handling file changes. It's a thin convenience wrapper over [`notify`](https://github.com/passcod/notify), allowing you to easily set callbacks for each path you want to watch.

Only the latest stable version of Rust is supported.

```rust
use hotwatch::{notify::event::ModifyKind, EventKind, Hotwatch};

let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
hotwatch.watch("war.png", |event: Event| {
    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
        println!("War has changed.");
    }
}).expect("failed to watch file!");
```

## Why should I use this instead of [`warmy`](https://github.com/phaazon/warmy)?

`warmy` is a more general solution for responding to resource changes. `hotwatch` is very simplistic and intends to be trivial to integrate.

I've never actually used `warmy`, though. It's probably awesome. I just know that `hotwatch` is really easy to use and has a sexy name.
