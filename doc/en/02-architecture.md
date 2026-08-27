# Cavi — Architecture

## Module layout

```
src/
├── lib.rs              entry point, wires up modules, wasm-bindgen extern (alert)
├── utils.rs             misc utilities (panic hook setup, etc.)
└── world.rs              re-exports world submodules
    world/
    ├── point.rs           WasmPoint — 2D vector (x, y) with vector-math ops
    ├── position.rs        WasmPosition — Verlet position state (curr / old)
    ├── node.rs            WasmNode — a single point mass in a wire
    └── wire.rs            WasmWire — a chain of nodes representing one cable
```

`WasmWorld` itself lives directly in `world.rs` and is the top-level simulation container.

## Core types

### `WasmPoint` (`world/point.rs`)
A minimal 2D vector type (`x: f32`, `y: f32`) with arithmetic operators, used everywhere positions/velocities/forces are represented.

### `WasmPosition` (`world/position.rs`)
Wraps the **Verlet integration** state for a node: current position (`curr`) and previous position (`old`). Velocity is implicit (`curr - old`), which is the standard trick used by Verlet integrators to avoid storing velocity explicitly while still supporting friction/damping.

### `WasmNode` (`world/node.rs`)
A single point mass in a wire:
- `position: WasmPosition`
- `velocity: WasmPoint` (explicit velocity, used by some helper/force-based methods in addition to the Verlet position)
- `fixed: bool` — pinned nodes (e.g. wire endpoints) never move during integration

Movement is advanced with `update_position(dt, friction, acceleration)`, which applies damped Verlet integration plus acceleration (gravity).

### `WasmWire` (`world/wire.rs`)
An ordered list of `WasmNode`s plus wire-level parameters:
- `radius` — visual/collision radius of the cable
- `link_target_distance` — the distance neighboring nodes try to maintain (a distance constraint, relaxed each physics step)
- `render_type` — `0` = straight segments, `1` = smoothed curve (Bezier, see below)

Per-wire responsibilities:
- `update(dt, friction, acceleration)` — integrates all nodes, then relaxes the distance constraint between each pair of neighboring nodes
- `check_wire_elements_collisions(response_coef)` — pushes a wire's own nodes apart if they get closer than the wire's diameter (self-collision)
- `check_mouse_collision(mouse, mouse_radius)` — pushes nodes away from the mouse pointer
- Structural editing: `add_node`, `add_node_at`, `remove_node`, `set_node_count` (redistributes nodes evenly between the fixed endpoints)

### `WasmWorld` (`world.rs`)
The simulation container and the main entry point exposed to JS:
- Owns all `WasmWire`s, the mouse position, and global parameters (`mouse_radius`, `pointer_radius`, `response_coef`, `friction`, `acceleration`).
- `update()` runs the physics loop for **all** wires (2 sub-iterations per call for stability: collision handling + constraint relaxation), then recomputes the render-ready point buffer for every wire.
- Owns `wire_data_buffer: Vec<f32>` — a flat buffer rebuilt every `update()` call and exposed to JS via `wire_data_ptr()` / `wire_data_len()` so the renderer can read WASM linear memory directly (**zero-copy**) instead of crossing the JS/WASM boundary per node.

## Physics step, per `update()` call

1. For 2 sub-steps:
   a. `check_mouse_collision` — repel nodes near the mouse
   b. `check_wire_elements_collisions` — repel a wire's nodes from each other (self-collision)
   c. `update(dt=0.3, friction, acceleration)` — Verlet-integrate positions, then relax the neighbor distance constraint
2. Rebuild `wire_data_buffer` for **every** wire, converting raw node positions into render-ready points according to `render_type`.

## Curve generation

Two render strategies, selected per-wire via `render_type`:

- **Segments (`0`)** — `nodes_to_segments`: node positions are used as-is (polyline).
- **Bezier (`1`, default)** — `catmull_to_bezier_points`: converts the Catmull-Rom spline implied by consecutive nodes into a sequence of cubic Bezier control points, producing a smooth curve through all nodes. This is the standard Catmull-Rom → Bezier conversion (each interior segment's two control points are derived from the neighboring node positions).

## Wire data buffer format

For each wire, the following is appended to `wire_data_buffer` (all `f32`):

```
[node_count, radius, render_type, path_length, ...path_data]
```

Where `path_data` is either a flat list of `(x, y)` segment points (`render_type == 0`) or a flat list of `(cp1x, cp1y, cp2x, cp2y, x, y)` Bezier tuples (`render_type == 1`). `path_length` is the number of `f32` values in `path_data` (i.e. `points.len() * 2`).

This buffer is designed to be read on the JS side as a single `Float32Array` view into WASM memory (see `cavijs`'s `Renderer.drawAllWires`), avoiding per-node FFI calls.

## Collision & constraints model

- **Distance constraint** (`WasmWire::update`): for each pair of neighboring nodes, nudges both toward/away from each other by half the delta between actual distance and `link_target_distance`, unless a node is `fixed`.
- **Self-collision** (`check_wire_elements_collisions`): if two neighboring nodes are closer than the wire's diameter, pushes them apart, scaled by `response_coef` (defaults to `0.0`, i.e. disabled, in `cavijs`'s `World` constructor).
- **Mouse collision** (`check_mouse_collision` / `check_collision`): pushes movable nodes outside a circle of radius `mouse_radius + wire.radius` around the mouse/pointer.

Note: `WasmWire::intersect()` (used to gate mouse collision) is currently a stub that always returns `true` — see the `TODO` in `wire.rs`.

## Tests

Unit tests live alongside `lib.rs` and are gated with `#[cfg(test)]`:
- `point_tests.rs` — `WasmPoint` vector math
- `node_tests.rs` — `WasmNode` integration/state behavior
- `wire_tests.rs` — `WasmWire` constraint/collision behavior

Browser-based integration tests live in `tests/web.rs` (run via `wasm-pack test --headless --firefox`).
