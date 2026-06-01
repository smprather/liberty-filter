# Repository Guidelines

## Project Structure & Module Organization

This repository contains tools for reducing and rewriting Liberty timing files.
The Rust implementation lives in `src/main.rs` and is built as the
`liberty_filter` binary. Helper scripts such as `strip_cell_underscores.py` and `main.py`
are at the repository root. Large Liberty fixtures are also root-level, for
example `generic80_ss_125c_1p116v_0p84v.lib`; avoid adding generated benchmark
outputs unless they are intentionally part of a test fixture.

## Build, Test, and Development Commands

- `cargo build --release --offline`: builds the optimized Rust binary using
  cached dependencies.
- `cargo run --release -- --in-file generic80_ss_125c_1p116v_0p84v.lib --out-file /tmp/out.lib`:
  runs the Rust filter locally.
- `cargo fmt --check`: verifies standard Rust formatting.
- `cargo check --offline`: verifies the crate using cached dependencies.
- `hyperfine --warmup 3 --runs 10 '<rust command>'`: measures runtime
  performance.

## Coding Style & Naming Conventions

Rust code uses the standard 2021 edition style: four-space indentation,
`snake_case` functions and variables, and `CamelCase` types. Run `cargo fmt`
before submitting Rust changes. Keep the parser byte-oriented where possible;
Liberty files are large, and unnecessary UTF-8 conversion can hurt throughput.
Python helpers should be small, executable scripts with `snake_case` names.

## Testing Guidelines

There is no formal test suite yet. For parser changes, test both an unfiltered
run and at least one filtered run on `generic80_ss_125c_1p116v_0p84v.lib.gz`,
for example `--filter-out-cells ^and2 --remove-comments`. Add focused unit
tests under Rust `#[cfg(test)]` modules when extracting reusable parsing
functions.

## Commit & Pull Request Guidelines

The existing history uses short imperative summaries, such as
`adding test liberty file.` Keep commits focused and describe the observable
change. Pull requests should include the command lines used for validation,
note any performance impact, and mention fixture files touched or regenerated.
Do not include large generated outputs unless the PR explains why they are
needed.

## Agent-Specific Instructions

Preserve user-created working tree changes. Prefer `rg` for searches and use
`/tmp` for benchmark outputs. Do not rewrite large Liberty fixtures unless the
task explicitly asks for it.
