# 0.2

- Updated to notify 4.0.
- `hotwatch::Event` is now merely an alias for `notify::DebouncedEvent`, as it's a very nice type with even nicer documentation.
- `Hotwatch::watch` has become more powerful, but potentially more surprising; when watching a directory, the handler will now receive events for all contents, recursing into subdirectories. However, if any of those directory contents have their own handlers, only the most specific applicable handle will be fired. You can read about this in the [documentation](https://francesca64.github.io/hotwatch/docs/hotwatch/struct.Hotwatch.html#method.watch).

# 0.1

🍾
