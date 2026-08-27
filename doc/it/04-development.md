# Cavi — Sviluppo e Build

## Prerequisiti

Questo è un progetto Rust → WebAssembly. Per compilarlo servono:

- **Toolchain Rust** (`rustc`/`cargo`) — non attualmente installata in questo ambiente
- **`wasm-pack`** — non attualmente installato in questo ambiente
- (opzionale) `cargo generate`, necessario solo per creare un *nuovo* progetto a partire da questo template

> Questi strumenti **non sono installati** nell'ambiente corrente. Non verrà installato nulla automaticamente — chiedere conferma prima di eseguire qualsiasi comando di installazione (es. `rustup`, `cargo install wasm-pack`, o un package manager di sistema).

## Build

```bash
wasm-pack build --target web
```

Compila `src/` in WebAssembly e genera una cartella `pkg/` contenente i binding JS/TS e il binario `.wasm` — questa `pkg/` è ciò che `cavijs` consuma come sua dipendenza npm `cavi`.

## Test

I test unitari (`point_tests.rs`, `node_tests.rs`, `wire_tests.rs`) vengono eseguiti con il normale `cargo test` una volta disponibile una toolchain Rust. I test di integrazione basati su browser (`tests/web.rs`) richiedono un browser headless:

```bash
wasm-pack test --headless --firefox
```

## Pubblicazione

```bash
wasm-pack publish
```

## Profilo di release

`Cargo.toml` imposta `opt-level = "s"` per le build di release, ottimizzando per una dimensione ridotta del codice (importante per un payload WASM distribuito ai browser).

## Note su questo ambiente

- `cargo` e `wasm-pack` non sono presenti nel `PATH` di questo workspace.
- La cartella di output `pkg/` non esiste attualmente nel repository (è output di build, tipicamente in gitignore).
- `cavijs/package.json` dipende da `"cavi": "^0.1.0"` — come questa dipendenza venga risolta (registro npm vs. collegamento locale alla `pkg/` di questo progetto) andrebbe confermato prima di provare a installare le dipendenze di `cavijs`.
