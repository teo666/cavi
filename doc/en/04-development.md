# Cavi — Development & Build

## Prerequisites

This is a Rust → WebAssembly project. To build it you need:

- **Rust toolchain** (`rustc`/`cargo`) — not currently installed in this environment
- **`wasm-pack`** — not currently installed in this environment
- (optional) `cargo generate`, only needed if scaffolding a *new* project from this template

> These tools are **not installed** in the current environment. Nothing will be installed automatically — ask before running any install command (e.g. `rustup`, `cargo install wasm-pack`, or a system package manager).

## Build

```bash
wasm-pack build --target web
```

This compiles `src/` to WebAssembly and generates a `pkg/` directory containing the JS/TS bindings and `.wasm` binary — this `pkg/` is what `cavijs` consumes as its `cavi` npm dependency.

## Test

Unit tests (`point_tests.rs`, `node_tests.rs`, `wire_tests.rs`) run as part of normal `cargo test` once a Rust toolchain is available. Browser-based integration tests (`tests/web.rs`) require a headless browser:

```bash
wasm-pack test --headless --firefox
```

## Publish

```bash
wasm-pack publish
```

## Release profile

`Cargo.toml` sets `opt-level = "s"` for release builds, optimizing for small code size (important for a WASM payload shipped to browsers).

## Notes on this environment

- `cargo` and `wasm-pack` are not present on `PATH` in this workspace.
- The `pkg/` output directory does not currently exist in the repo (it's build output, typically gitignored).
- `cavijs/package.json` depends on `"cavi": "^0.1.0"` — how this resolves (npm registry vs. a local link to this project's `pkg/`) should be confirmed before attempting to install `cavijs`'s dependencies.
