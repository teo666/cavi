# Cavi — Development & Build

## Prerequisites

This is a Rust → WebAssembly project. To build it you need:

- **Rust toolchain** (`rustc`/`cargo`)
- **`wasm-pack`**

## Build

```bash
wasm-pack build --target web
```

This compiles `src/` to WebAssembly and generates a `pkg/` directory containing the JS/TS bindings and `.wasm` binary — this `pkg/` is what `cavijs` consumes as its `cavi` npm dependency.

## Test

Unit tests (`point_tests.rs`, `node_tests.rs`, `wire_tests.rs`, `world_tests.rs`) run with `cargo test`. Browser-based integration tests (`tests/web.rs`) require a headless browser:

```bash
wasm-pack test --headless --firefox
```

## Publish

```bash
wasm-pack publish
```

## Release profile

`Cargo.toml` sets `opt-level = "s"` for release builds, optimizing for small code size (important for a WASM payload shipped to browsers).

## Developing with `cavijs`

`pkg/` is build output (gitignored). To try engine changes in `cavijs` before publishing, point its dependency at the local build — `"cavi": "link:../cavi/pkg"` in `cavijs/package.json`, then `pnpm install` — and rebuild with `wasm-pack build --target web` after each change. Switch it back to a published version before merging `cavijs`.
