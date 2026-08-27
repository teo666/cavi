# Cavi — API Reference

All types below are exposed to JavaScript/TypeScript via `#[wasm_bindgen]`. This reference groups methods by type; see [`cavijs`'s API doc](../../../cavijs/doc/en/03-api.md) for the higher-level TypeScript wrapper built on top of this.

## `WasmWorld`

The main simulation container. Constructed with `new WasmWorld()`.

**Wire management**
| Method | Description |
|---|---|
| `add_wire_debug()` | Adds a default 30-node horizontal debug wire |
| `add_wire(xs, ys, xe, ye, radius)` | Adds a wire between two points; node count is derived automatically |
| `add_wire_with_count(xs, ys, xe, ye, node_count, link_target, radius, render_type)` | Adds a wire with explicit node count, constraint distance, radius, and render type (`0`=segments, `1`=bezier) |
| `delete_wire(index) -> bool` | Removes a wire by index |
| `wire_count() -> usize` | Number of wires |
| `get_wire(index) -> Option<WasmWire>` | Returns a **copy** of a wire |

**Per-wire node access**
| Method | Description |
|---|---|
| `get_wire_node(wire_idx, node_idx) -> Option<WasmNode>` | Copy of a node |
| `get_wire_node_x/y(wire_idx, node_idx) -> f32` | Node coordinates |
| `get_wire_node_count(wire_idx) -> usize` | Node count for a wire |
| `add_wire_node(wire_idx, x, y, fixed)` | Append a node |
| `add_wire_node_at(wire_idx, node_idx, x, y, fixed)` | Insert a node at index |
| `remove_wire_node(wire_idx, node_idx) -> bool` | Remove a node |
| `set_wire_node_count(wire_idx, node_count)` | Resize a wire, redistributing nodes evenly |
| `set_wire_node_fixed(wire_idx, node_idx, fixed)` | Pin/unpin a node |
| `set_wire_node_position(wire_idx, node_idx, x, y)` | Move a node |
| `set_wire_start(wire_idx, x, y)` / `set_wire_end(wire_idx, x, y)` | Move the fixed endpoints |
| `get_wire_radius(wire_idx) -> f32` / `set_wire_radius(wire_idx, radius)` | Wire thickness |

**Simulation**
| Method | Description |
|---|---|
| `update()` | Advances physics for all wires and rebuilds the render buffer |
| `set_mouse(x, y)` | Sets pointer position for collision |
| `wire_optimal_length(start_x, start_y, end_x, end_y, radius) -> usize` (static) | Suggests a node count for a wire given endpoints and radius |

**Render buffer (zero-copy)**
| Method | Description |
|---|---|
| `wire_data_ptr() -> *const f32` | Pointer into WASM linear memory for the render buffer |
| `wire_data_len() -> usize` | Buffer length, in `f32` elements |

See [Architecture → Wire data buffer format](./02-architecture.md#wire-data-buffer-format) for the layout.

**Global configuration (getters/setters)**
| Parameter | Default | Description |
|---|---|---|
| `mouse_radius` | `40.0` | Radius of pointer repulsion |
| `pointer_radius` | `20.0` | Additional collision radius used elsewhere |
| `response_coef` | `0.0` | Strength of wire self-collision response |
| `friction` | `0.95` | Verlet damping factor |
| `acceleration` | `(0.0, 10.0)` | Global acceleration (gravity), `get_acceleration_x/y()` also available |

## `WasmWire`

Constructed via `WasmWire::new()` (30-node default wire) or the static factories `new_wire(...)` / `new_with_count(...)` (mirrors `WasmWorld::add_wire[_with_count]`).

| Method | Description |
|---|---|
| `node_count() -> usize` | Number of nodes |
| `get_node(index) -> Option<WasmNode>` | Copy of a node |
| `get_node_x/y(index) -> f32` | Node coordinates |
| `get_radius() -> f32` / `set_radius(radius)` | Also updates `link_target_distance = radius * 3.0` |
| `add_node(x, y, fixed)` / `add_node_at(index, x, y, fixed)` / `remove_node(index) -> bool` | Structural edits; `remove_node` refuses to drop below 2 nodes |
| `get_render_type() -> u8` / `set_render_type(render_type)` | `0` = segments, `1` = bezier |
| `set_node_count(new_count)` | Rebuilds the node list, keeping endpoints fixed, evenly spacing the rest |
| `get_start()` / `get_end() -> WasmPoint` | Endpoint positions |
| `set_start(x, y)` / `set_end(x, y)` | Move endpoints |
| `set_node_fixed(node_idx, fixed)` / `set_node_position(node_idx, x, y)` | Per-node edits |
| `length() -> f32` | Sum of segment distances (polyline length) |
| `check_collision(point, pointer_radius)` | Pushes movable nodes outside `radius + pointer_radius` of `point` |
| `update(dt, friction, acceleration)` | Integrates nodes, then relaxes neighbor distance constraint |
| `check_wire_elements_collisions(response_coef)` | Self-collision |
| `check_mouse_collision(mouse, mouse_radius)` | Pointer repulsion |
| `invalidate()` | Resets the internal iteration counter |
| `optimal_length(start, end, node_radius) -> usize` (static) | Node-count heuristic used by `WasmWorld::wire_optimal_length` |

Public fields: `iterations: u32`, `radius: f32`, `link_target_distance: f32`, `render_type: u8`.

## `WasmNode`

Constructed via `new(position, velocity, fixed)`, or the convenience constructors `new_no_vel(x, y, fixed)`, `zero()`, `new_fixed(x, y)`.

| Method | Description |
|---|---|
| `get_x() / get_y() -> f32` | Current position |
| `get_velocity_x/y() -> f32` / `get_velocity() -> WasmPoint` | Explicit velocity field |
| `get_fixed() -> bool` / `is_fixed()` / `is_movable()` | Pinned state |
| `set_position(x, y)` / `set_velocity(vx, vy)` / `set_fixed(fixed)` | Mutators |
| `make_fixed()` / `make_movable()` | Convenience pin toggles |
| `get_position() -> WasmPoint` | Current position as a point |
| `get_speed() -> f32` | Magnitude of `velocity` |
| `apply_force(fx, fy)` / `apply_force_point(force)` | Adds to `velocity` (no-op if fixed) |
| `update_position(dt, friction, acceleration)` | Damped Verlet integration step (no-op if fixed) |
| `update_with_acceleration(acceleration, dt, friction)` | Alternate integration entry point via `WasmPosition::integrate` |
| `translate(dx, dy)` | Direct position delta |
| `reset_velocity()` | Zeroes velocity and Verlet history |
| `distance_to(other)` / `distance_squared_to(other) -> f32` | Distance helpers |
| `print() -> String` | Debug string (also via `Display`) |

Public fields: `position: WasmPosition`, `velocity: WasmPoint`, `fixed: bool`.

## `WasmPoint` / `WasmPosition`

Low-level vector/position types used throughout the API above (arithmetic operators, `zero()`, distance helpers). See `world/point.rs` and `world/position.rs` for the full method list — these are mostly consumed internally and via the `get_position()` / `WasmPoint` fields on `WasmNode`/`WasmWire`.
