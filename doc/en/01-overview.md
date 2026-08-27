# Cavi — Overview

`cavi` is a Rust library compiled to **WebAssembly** (via `wasm-bindgen` / `wasm-pack`) that implements a 2D physics simulation for flexible cables ("wires"). It is designed to be consumed from the browser by the companion TypeScript project [`cavijs`](../../../cavijs/doc/en/01-overview.md), which handles rendering and DOM/UI concerns.

## Purpose

The engine simulates hanging/draggable cables made of connected points ("nodes"), using a **Verlet integration** scheme with:

- Gravity / configurable acceleration
- Friction (damping)
- Distance constraints between neighboring nodes (keeps the cable's segments close to a target length)
- Self-collision between a wire's own nodes
- Collision/repulsion against a mouse pointer

The simulation only produces **numeric data** (node positions, curve control points). It intentionally does **not** know anything about rendering, colors, or DOM — that responsibility is left entirely to the JavaScript/TypeScript side, so the renderer can be swapped or extended freely.

## Why WebAssembly

Physics integration and collision checks run every frame and scale with the number of wires/nodes; doing this in Rust/WASM keeps it fast and avoids allocations in hot paths where possible (see the `wire_data_buffer` in `WasmWorld`, reused across frames and exposed to JS as a zero-copy `Float32Array` view).

## Project status

This repository was bootstrapped from the [`rustwasm/wasm-pack-template`](https://github.com/rustwasm/wasm-pack-template) — the root `README.md` still contains the template's generic instructions rather than project-specific documentation; this `doc/` folder is the actual project documentation.

## Relationship to `cavijs`

- `cavi` (this project): physics simulation core, compiled to a WASM package, published/consumed as the npm package `cavi`.
- `cavijs`: TypeScript wrapper, canvas renderer, and Web Components (`Cavi`, `World`, `Wire`, `Node`, `Renderer`, `cavi-jack`, `cavi-plug`) built on top of it.

See also:
- [Architecture](./02-architecture.md)
- [API reference](./03-api.md)
- [Development / build](./04-development.md)
