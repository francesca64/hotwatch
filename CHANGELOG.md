# Unreleased

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
