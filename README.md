# hotwatch

[![Clippy Linting Result](https://clippy.bashy.io/github/francesca64/hotwatch/master/badge.svg)](https://clippy.bashy.io/github/francesca64/hotwatch/master/log)

*hotwatch* is a Rust library for conveniently watching and handling file changes.

Nightly Rust is required, since I used the box keyword a few times. Sorry.

Real documentation is coming soon; I promise. In the mean time, have an example:

```rust
use hotwatch::Hotwatch;

let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
hotwatch.watch("war.png", |e: hotwatch::Event| {
  if e.changed() {
    println!("War has changed.");
  }
}).expect("Failed to watch file!");
```
