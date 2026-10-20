# cavi

A 2D physics engine for flexible cables, written in Rust and compiled to WebAssembly.

Cables ("wires") are chains of nodes simulated with Verlet integration: gravity, friction, distance
constraints between neighbouring nodes, self-collision, and repulsion from the mouse pointer. The
engine produces numbers only — node positions and ready-to-draw curve points — and knows nothing
about rendering or the DOM. Its companion project, [`cavijs`](https://github.com/teo666/cavijs),
draws the cables and provides the web components (worlds, jacks, plugs) on top of it.

## Highlights

- **Zero-copy rendering data.** After each step the engine writes every wire's segments or Bézier
  control points into one flat `f32` buffer, which JavaScript reads directly from WASM memory.
- **Frame-rate independent.** `step()` advances the simulation by a fixed amount of time;
  `build_render_data(alpha)` draws each cable between the last two steps, so a host stepping at a
  fixed rate gets the same physics on every display and a new position on every frame of a fast one.
- **Interaction aware.** A held wire (one being dragged) is not pushed by the pointer that drags it,
  and a dropped cable end is thrown with the speed of the hand, not of the whole drag.
- **Cheap at rest.** Wires that stop moving go to sleep and are skipped until something touches them.
- **Cables, not just chains.** A wire can resist bending (`set_wire_bend_stiffness`, 0 to 1): instead
  of hanging straight down from a point and folding where the rope runs out, it curves all along.
  Each end can be given the direction it leaves its anchor by (`set_wire_end_direction`), like a plug
  out of a jack. Off by default.
- **Tunable.** Time step, sub-steps, constraint iterations, friction, gravity, pointer radii and
  sleep thresholds are all configurable.

## Usage

```js
import init, { WasmWorld } from 'cavi';

const wasm = await init();
const world = new WasmWorld();
// From (100, 100) to (400, 100): 20 nodes, 18 units apart, radius 4, Bézier rendering.
world.add_wire_with_count(100, 100, 400, 100, 20, 18, 4, 1);

function frame() {
  world.update(); // step() + build_render_data(1.0)
  const data = new Float32Array(wasm.memory.buffer, world.wire_data_ptr(), world.wire_data_len());
  // per wire: [node_count, radius, render_type, path_length, ...path points]
  requestAnimationFrame(frame);
}
```

## Build and test

```bash
cargo test
wasm-pack build --target web    # output in pkg/
```

## Documentation

- English: [overview](doc/en/01-overview.md), [architecture](doc/en/02-architecture.md),
  [API](doc/en/03-api.md), [development](doc/en/04-development.md)
- Italiano: [panoramica](doc/it/01-overview.md), [architettura](doc/it/02-architecture.md),
  [API](doc/it/03-api.md), [sviluppo](doc/it/04-development.md)

## License

Dual-licensed under [Apache 2.0](LICENSE_APACHE) or [MIT](LICENSE_MIT), at your option.
