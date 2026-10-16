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
| `get_wire_node_render_x/y(wire_idx, node_idx) -> f32` | Where the node is drawn by the last `build_render_data` (interpolated) |
| `get_wire_node_fixed(wire_idx, node_idx) -> bool` | Whether a node is fixed, without copying it out |
| `get_wire_node_count(wire_idx) -> usize` | Node count for a wire |
| `add_wire_node(wire_idx, x, y, fixed)` | Append a node |
| `add_wire_node_at(wire_idx, node_idx, x, y, fixed)` | Insert a node at index |
| `remove_wire_node(wire_idx, node_idx) -> bool` | Remove a node |
| `set_wire_node_count(wire_idx, node_count)` | Resize a wire, redistributing nodes evenly |
| `set_wire_node_fixed(wire_idx, node_idx, fixed)` | Pin/unpin a node. Releasing a pinned node throws it with the velocity of its last movement (see [Moving nodes](#moving-nodes-from-outside)) |
| `set_wire_node_position(wire_idx, node_idx, x, y)` | Move a node from outside the simulation (see [Moving nodes](#moving-nodes-from-outside)) |
| `set_wire_start(wire_idx, x, y)` / `set_wire_end(wire_idx, x, y)` | Move the fixed endpoints |
| `get_wire_radius(wire_idx) -> f32` / `set_wire_radius(wire_idx, radius)` | Wire thickness (does not change the distance between nodes) |
| `get_wire_link_distance(wire_idx) -> f32` / `set_wire_link_distance(wire_idx, distance)` | Rest distance between neighbouring nodes |
| `get_wire_render_type(wire_idx) -> u8` / `set_wire_render_type(wire_idx, render_type)` | Render type of a wire (`0` = segments, `1` = bezier), used by the next `build_render_data` |
| `is_wire_sleeping(wire_idx) -> bool` | Whether a wire is asleep (see [Sleeping](#sleeping)) |

**Simulation**
| Method | Description |
|---|---|
| `update()` | `step()` followed by `build_render_data(1.0)` |
| `step()` | Advances physics for all wires by one step (`substeps` × `time_step`); does not touch the render buffer |
| `build_render_data(alpha)` | Rebuilds the render buffer, drawing each movable node `alpha` (0..1) of the way from its position before the last step to its current one; fixed nodes are drawn where they are. Call it on every frame, even when no step ran |
| `set_mouse(x, y)` | Sets pointer position for collision |
| `set_held_wire(wire_idx)` / `release_held_wire()` / `get_held_wire() -> Option<usize>` | The wire the user is holding (dragging one of its ends): see [Held wire](#held-wire) |
| `sleeping_wire_count() -> usize` / `wake_all()` | Sleeping wires, and waking them all |
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
| `pointer_radius` | `20.0` | Radius within which the held point pushes the other wires while a wire is held |
| `response_coef` | `0.0` | Strength of wire self-collision response |
| `friction` | `0.95` | Verlet damping factor |
| `acceleration` | `(0.0, 10.0)` | Global acceleration (gravity), `get_acceleration_x/y()` also available |
| `time_step` | `0.3` | Simulated time per sub-step. Larger is faster but less stable |
| `substeps` | `2` | Sub-steps per `step()` (at least 1). Each advances time by `time_step`, so this changes speed as well as accuracy |
| `constraint_iterations` | `1` | Link relaxation passes per sub-step (at least 1): more make cables stiffer, without changing speed |
| `render_alpha` | `1.0` | Read-only: the `alpha` of the last `build_render_data` |
| `sleep_enabled` | `true` | Put still wires to sleep (turning it off wakes them all) |
| `sleep_threshold` | `0.01` | Largest movement of a node in a step that counts as still |
| `sleep_steps` | `30` | Still steps in a row before a wire sleeps (at least 1) |

**Frame-rate independent stepping.** The step has a fixed time delta, so call `step()` at a fixed rate (e.g. 60 per second, with an accumulator) and `build_render_data(accumulator / step_duration)` on every frame: physics behaves the same on every display, and a display faster than the step rate still draws a new position on every frame.

## `WasmWire`

Constructed via `WasmWire::new()` (30-node default wire) or the static factories `new_wire(...)` / `new_with_count(...)` (mirrors `WasmWorld::add_wire[_with_count]`). The factories place the nodes evenly on the straight line between the two ends (fixed), so a new wire starts where it is drawn instead of flying in from a fixed point.

| Method | Description |
|---|---|
| `node_count() -> usize` | Number of nodes |
| `get_node(index) -> Option<WasmNode>` | Copy of a node |
| `get_node_x/y(index) -> f32` | Node coordinates |
| `get_radius() -> f32` / `set_radius(radius)` | Thickness only |
| `get_link_distance() -> f32` / `set_link_distance(distance)` | Rest distance between neighbouring nodes (`link_target_distance`) |
| `add_node(x, y, fixed)` / `add_node_at(index, x, y, fixed)` / `remove_node(index) -> bool` | Structural edits; `remove_node` refuses to drop below 2 nodes |
| `get_render_type() -> u8` / `set_render_type(render_type)` | `0` = segments, `1` = bezier |
| `set_node_count(new_count)` | Rebuilds the node list, keeping endpoints fixed, evenly spacing the rest |
| `get_start()` / `get_end() -> WasmPoint` | Endpoint positions |
| `set_start(x, y)` / `set_end(x, y)` | Move endpoints |
| `set_node_fixed(node_idx, fixed)` / `set_node_position(node_idx, x, y)` | Per-node edits; `set_node_position` places the node like `WasmNode::move_to` |
| `is_node_fixed(index) -> bool` | Whether a node is fixed |
| `is_sleeping() -> bool` | Whether the wire is asleep |
| `length() -> f32` | Sum of segment distances (polyline length) |
| `check_collision(point, pointer_radius)` | Pushes movable nodes outside `radius + pointer_radius` of `point` |
| `update(dt, friction, acceleration)` | Integrates nodes, then relaxes neighbor distance constraint once |
| `update_with_iterations(dt, friction, acceleration, iterations)` | Same, relaxing the constraint `iterations` times |
| `is_node_fixed(index) -> bool` | Whether a node is fixed |
| `check_wire_elements_collisions(response_coef)` | Self-collision |
| `check_mouse_collision(mouse, mouse_radius)` | Pointer repulsion |
| `invalidate()` | Wakes the wire up and restarts its count of still steps |
| `optimal_length(start, end, node_radius) -> usize` (static) | Node-count heuristic used by `WasmWorld::wire_optimal_length` |

Public fields: `radius: f32`, `link_target_distance: f32`, `render_type: u8`.

### Held wire

While the user drags one end of a wire, the host calls `set_held_wire(index)` (and `release_held_wire()` on release). The mouse, which the host keeps on the dragged end, then:
- does not push the held wire at all — otherwise it would push that wire's own nodes away from the dragged end and stretch it;
- pushes the other wires within `pointer_radius` instead of `mouse_radius`.

With nothing held, the mouse pushes every wire within `mouse_radius`. The held index follows the wire when an earlier wire is deleted, and is cleared when the held wire itself is deleted.

### Sleeping

A wire whose nodes all moved less than `sleep_threshold` in each of `sleep_steps` steps in a row goes to sleep: `step()` skips it, and its residual velocity is dropped. It wakes up when anything can set it in motion again: the mouse comes within reach of its bounding box, a node is moved, fixed or released, nodes are added or removed, its radius or link distance changes, or a global parameter changes (gravity, friction, response coefficient, time step, sub-steps, constraint iterations). Sleeping wires are still drawn.

### Moving nodes from outside

`set_wire_node_position` / `set_wire_start` / `set_wire_end` (and `WasmNode::move_to`) place a node from outside the simulation:
- a **free** node is teleported, with no velocity;
- a **fixed** node (a dragged or plugged end) only has its current position set, and its previous position follows it at every integration, so it never builds up the whole distance moved while fixed as velocity. When it is released with `set_wire_node_fixed(…, false)`, it is thrown with the distance it moved over the most recent step, spread over the sub-steps; a node held still just falls.

Collisions and constraints inside the engine keep using `WasmNode::set_position`, which moves only the current position, so their corrections still carry momentum (Verlet).

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
| `update_position(dt, friction, acceleration)` | Damped Verlet integration step (a fixed node only has its previous position synced to its current one) |
| `set_position(x, y)` | Sets the current position only (the move counts as velocity) — used by collisions and constraints |
| `move_to(x, y)` | Places the node from outside the simulation (see [Moving nodes](#moving-nodes-from-outside)) |
| `update_with_acceleration(acceleration, dt, friction)` | Alternate integration entry point via `WasmPosition::integrate` |
| `translate(dx, dy)` | Direct position delta |
| `reset_velocity()` | Zeroes velocity and Verlet history |
| `distance_to(other)` / `distance_squared_to(other) -> f32` | Distance helpers |
| `print() -> String` | Debug string (also via `Display`) |

Public fields: `position: WasmPosition`, `velocity: WasmPoint`, `fixed: bool`.

## `WasmPoint` / `WasmPosition`

Low-level vector/position types used throughout the API above (arithmetic operators, `zero()`, distance helpers). See `world/point.rs` and `world/position.rs` for the full method list — these are mostly consumed internally and via the `get_position()` / `WasmPoint` fields on `WasmNode`/`WasmWire`.
