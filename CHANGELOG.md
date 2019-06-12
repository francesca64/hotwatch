# Unreleased

# Version 0.4.2 (2019-06-12)

- Re-export `notify`.
- Implemented `Debug` for `Hotwatch`.
- Added `Hotwatch::new_with_custom_delay`.
- Added `Hotwatch::unwatch`.
- The background thread will now stop once `Hotwatch` is dropped.
- Updated docs and added an example.
- Removed dependency on `derive_more` and `parking_lot`.

# Version 0.4.1 (2019-05-31)

- Corrected doc links.

# Version 0.4.0 (2019-05-30)

This release removes the claim of compatability with Rust 1.24, as a patch update to `notify` changes the minimum requirement to 1.26.

`hotwatch` 0.3 can still be used with Rust 1.24 if you pin the `notify` dependency to `4.0.6`.

- Only the latest stable release of Rust is guaranteed to be compatible.
- `hotwatch::Error` now implements `std::error::Error`.

# Version 0.3.1 (2018-06-15)

This release makes `hotwatch` seem significantly more like a legitimate crate.

- `hotwatch` no longer requires nightly Rust! Minimum supported version is 1.24.
- Uses `log` instead of obnoxiously using `println!`.
- Updated dependencies.
- Relicensed as dual Apache-2.0/MIT.

# Version 0.3.0 (2017-07-22)

This release includes a non-breaking API change and a potentially breaking behavior change.

- `Hotwatch::watch` now accepts any path type that satisfies `AsRef<Path>`.
- Paths are automatically canonicalized. This is to prevent surprising behavior with handler matching. As a result of this, the paths enclosed in `hotwatch::Event` variants are now absolute, which can potentially break existing applications.

# Version 0.2.0 (2017-07-02)

This release includes significant breaking API changes.

- Updated to notify 4.0.
- `hotwatch::Event` is now merely an alias for `notify::DebouncedEvent`, as it's a very nice type with even nicer documentation.
- `Hotwatch::watch` has become more powerful, but potentially more surprising; when watching a directory, the handler will now receive events for all contents, recursing into subdirectories. However, if any of those directory contents have their own handlers, only the most specific applicable handle will be fired. You can read about this in the [documentation](https://francesca64.github.io/hotwatch/docs/hotwatch/struct.Hotwatch.html#method.watch).

# Version 0.1.0 (2016-11-12)

🍾
