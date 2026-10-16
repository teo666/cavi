# Cavi — Sviluppo e Build

## Prerequisiti

Questo è un progetto Rust → WebAssembly. Per compilarlo servono:

- **Toolchain Rust** (`rustc`/`cargo`)
- **`wasm-pack`**

## Build

```bash
wasm-pack build --target web
```

Compila `src/` in WebAssembly e genera una cartella `pkg/` contenente i binding JS/TS e il binario `.wasm` — questa `pkg/` è ciò che `cavijs` consuma come sua dipendenza npm `cavi`.

## Test

I test unitari (`point_tests.rs`, `node_tests.rs`, `wire_tests.rs`, `world_tests.rs`) si eseguono con `cargo test`. I test di integrazione basati su browser (`tests/web.rs`) richiedono un browser headless:

```bash
wasm-pack test --headless --firefox
```

## Pubblicazione

```bash
wasm-pack publish
```

## Profilo di release

`Cargo.toml` imposta `opt-level = "s"` per le build di release, ottimizzando per una dimensione ridotta del codice (importante per un payload WASM distribuito ai browser).

## Sviluppo insieme a `cavijs`

`pkg/` è output di build (in gitignore). Per provare le modifiche al motore in `cavijs` prima di pubblicarle, punta la sua dipendenza alla build locale (`"cavi": "link:../cavi/pkg"` in `cavijs/package.json`, poi `pnpm install`) e ricompila con `wasm-pack build --target web` dopo ogni modifica. Prima di fare il merge di `cavijs`, rimetti la dipendenza su una versione pubblicata.
