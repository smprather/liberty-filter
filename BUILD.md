# Build

This project builds one Rust executable: `liberty_filter`.

## Prerequisites

- Rust 2021 toolchain
- Cargo
- Cached crates when building with `--offline`

## Build

```sh
cargo build --release --offline
```

The optimized executable is:

```sh
target/release/liberty_filter
```

For development builds:

```sh
cargo build
```

## Run

```sh
cargo run --release -- --in-file generic80_ss_125c_1p116v_0p84v.lib.gz --out-file /tmp/out.lib.gz
```

Equivalent direct binary invocation after a release build:

```sh
target/release/liberty_filter --in-file generic80_ss_125c_1p116v_0p84v.lib.gz --out-file /tmp/out.lib.gz
```

Useful options:

```sh
target/release/liberty_filter \
  --in-file generic80_ss_125c_1p116v_0p84v.lib.gz \
  --out-file /tmp/out.lib.gz \
  --filter-out-cells '^and2' \
  --remove-comments
```

Set `LIBERTY_FILTER_BUF_SIZE` to override the default I/O buffer size.

## Check

Run Rust formatting and compile checks before committing:

```sh
cargo fmt --check
cargo check --offline
```

Run the tool against the included fixture:

```sh
cargo run --release --offline -- \
  --in-file generic80_ss_125c_1p116v_0p84v.lib.gz \
  --out-file /tmp/liberty_filter.lib.gz
```

## Release Notes

Keep generated outputs in `/tmp` unless they are intentional fixtures.
