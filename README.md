# liberty-filter

Rust command-line tool for filtering unneeded data out of Liberty timing files.

Build the executable with Cargo:

```sh
cargo build --release --offline
```

Dependencies are vendored in `vendor/`, so offline Cargo builds should not need
network access.

The binary is written to `target/release/liberty_filter`.

See [BUILD.md](BUILD.md) for build and validation commands.
