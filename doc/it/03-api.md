# Cavi — Riferimento API

Tutti i tipi seguenti sono esposti a JavaScript/TypeScript tramite `#[wasm_bindgen]`. Questo riferimento raggruppa i metodi per tipo; vedi la [documentazione API di `cavijs`](../../../cavijs/doc/it/03-api.md) per il wrapper TypeScript di livello più alto costruito sopra a questa.

## `WasmWorld`

Il contenitore principale della simulazione. Costruito con `new WasmWorld()`.

**Gestione dei cavi**
| Metodo | Descrizione |
|---|---|
| `add_wire_debug()` | Aggiunge un cavo orizzontale di debug con 30 nodi |
| `add_wire(xs, ys, xe, ye, radius)` | Aggiunge un cavo tra due punti; il numero di nodi è derivato automaticamente |
| `add_wire_with_count(xs, ys, xe, ye, node_count, link_target, radius, render_type)` | Aggiunge un cavo con numero di nodi, distanza di vincolo, raggio e tipo di rendering espliciti (`0`=segmenti, `1`=bezier) |
| `delete_wire(index) -> bool` | Rimuove un cavo per indice |
| `wire_count() -> usize` | Numero di cavi |
| `get_wire(index) -> Option<WasmWire>` | Ritorna una **copia** di un cavo |

**Accesso ai nodi per cavo**
| Metodo | Descrizione |
|---|---|
| `get_wire_node(wire_idx, node_idx) -> Option<WasmNode>` | Copia di un nodo |
| `get_wire_node_x/y(wire_idx, node_idx) -> f32` | Coordinate del nodo |
| `get_wire_node_count(wire_idx) -> usize` | Numero di nodi di un cavo |
| `add_wire_node(wire_idx, x, y, fixed)` | Aggiunge un nodo in coda |
| `add_wire_node_at(wire_idx, node_idx, x, y, fixed)` | Inserisce un nodo a un indice |
| `remove_wire_node(wire_idx, node_idx) -> bool` | Rimuove un nodo |
| `set_wire_node_count(wire_idx, node_count)` | Ridimensiona un cavo, ridistribuendo i nodi uniformemente |
| `set_wire_node_fixed(wire_idx, node_idx, fixed)` | Fissa/rilascia un nodo |
| `set_wire_node_position(wire_idx, node_idx, x, y)` | Sposta un nodo |
| `set_wire_start(wire_idx, x, y)` / `set_wire_end(wire_idx, x, y)` | Sposta gli estremi fissi |
| `get_wire_radius(wire_idx) -> f32` / `set_wire_radius(wire_idx, radius)` | Spessore del cavo |

**Simulazione**
| Metodo | Descrizione |
|---|---|
| `update()` | Avanza la fisica per tutti i cavi e ricostruisce il buffer di rendering |
| `set_mouse(x, y)` | Imposta la posizione del puntatore per le collisioni |
| `wire_optimal_length(start_x, start_y, end_x, end_y, radius) -> usize` (statico) | Suggerisce un numero di nodi per un cavo dati gli estremi e il raggio |

**Buffer di rendering (a copia zero)**
| Metodo | Descrizione |
|---|---|
| `wire_data_ptr() -> *const f32` | Puntatore alla memoria lineare WASM per il buffer di rendering |
| `wire_data_len() -> usize` | Lunghezza del buffer, in elementi `f32` |

Vedi [Architettura → Formato del buffer dati dei cavi](./02-architecture.md#formato-del-buffer-dati-dei-cavi) per il layout.

**Configurazione globale (getter/setter)**
| Parametro | Default | Descrizione |
|---|---|---|
| `mouse_radius` | `40.0` | Raggio di repulsione del puntatore |
| `pointer_radius` | `20.0` | Raggio di collisione aggiuntivo usato altrove |
| `response_coef` | `0.0` | Intensità della risposta alla self-collision |
| `friction` | `0.95` | Fattore di smorzamento Verlet |
| `acceleration` | `(0.0, 10.0)` | Accelerazione globale (gravità), disponibili anche `get_acceleration_x/y()` |

## `WasmWire`

Costruito tramite `WasmWire::new()` (cavo di default a 30 nodi) o le factory statiche `new_wire(...)` / `new_with_count(...)` (rispecchiano `WasmWorld::add_wire[_with_count]`).

| Metodo | Descrizione |
|---|---|
| `node_count() -> usize` | Numero di nodi |
| `get_node(index) -> Option<WasmNode>` | Copia di un nodo |
| `get_node_x/y(index) -> f32` | Coordinate del nodo |
| `get_radius() -> f32` / `set_radius(radius)` | Aggiorna anche `link_target_distance = radius * 3.0` |
| `add_node(x, y, fixed)` / `add_node_at(index, x, y, fixed)` / `remove_node(index) -> bool` | Modifiche strutturali; `remove_node` rifiuta di scendere sotto 2 nodi |
| `get_render_type() -> u8` / `set_render_type(render_type)` | `0` = segmenti, `1` = bezier |
| `set_node_count(new_count)` | Ricostruisce la lista dei nodi, mantenendo fissi gli estremi, distribuendo uniformemente il resto |
| `get_start()` / `get_end() -> WasmPoint` | Posizioni degli estremi |
| `set_start(x, y)` / `set_end(x, y)` | Sposta gli estremi |
| `set_node_fixed(node_idx, fixed)` / `set_node_position(node_idx, x, y)` | Modifiche per singolo nodo |
| `length() -> f32` | Somma delle distanze dei segmenti (lunghezza della polilinea) |
| `check_collision(point, pointer_radius)` | Spinge i nodi mobili fuori da `radius + pointer_radius` rispetto a `point` |
| `update(dt, friction, acceleration)` | Integra i nodi, poi rilassa il vincolo di distanza tra vicini |
| `check_wire_elements_collisions(response_coef)` | Self-collision |
| `check_mouse_collision(mouse, mouse_radius)` | Repulsione dal puntatore |
| `invalidate()` | Azzera il contatore interno di iterazioni |
| `optimal_length(start, end, node_radius) -> usize` (statico) | Euristica sul numero di nodi usata da `WasmWorld::wire_optimal_length` |

Campi pubblici: `iterations: u32`, `radius: f32`, `link_target_distance: f32`, `render_type: u8`.

## `WasmNode`

Costruito tramite `new(position, velocity, fixed)`, oppure i costruttori di comodo `new_no_vel(x, y, fixed)`, `zero()`, `new_fixed(x, y)`.

| Metodo | Descrizione |
|---|---|
| `get_x() / get_y() -> f32` | Posizione corrente |
| `get_velocity_x/y() -> f32` / `get_velocity() -> WasmPoint` | Campo velocità esplicito |
| `get_fixed() -> bool` / `is_fixed()` / `is_movable()` | Stato di fissaggio |
| `set_position(x, y)` / `set_velocity(vx, vy)` / `set_fixed(fixed)` | Mutatori |
| `make_fixed()` / `make_movable()` | Scorciatoie per il fissaggio |
| `get_position() -> WasmPoint` | Posizione corrente come punto |
| `get_speed() -> f32` | Modulo di `velocity` |
| `apply_force(fx, fy)` / `apply_force_point(force)` | Somma a `velocity` (nessun effetto se fissato) |
| `update_position(dt, friction, acceleration)` | Passo di integrazione Verlet smorzata (nessun effetto se fissato) |
| `update_with_acceleration(acceleration, dt, friction)` | Punto di ingresso alternativo per l'integrazione tramite `WasmPosition::integrate` |
| `translate(dx, dy)` | Delta di posizione diretto |
| `reset_velocity()` | Azzera velocità e storico Verlet |
| `distance_to(other)` / `distance_squared_to(other) -> f32` | Funzioni di distanza |
| `print() -> String` | Stringa di debug (anche via `Display`) |

Campi pubblici: `position: WasmPosition`, `velocity: WasmPoint`, `fixed: bool`.

## `WasmPoint` / `WasmPosition`

Tipi vettore/posizione di basso livello usati in tutta l'API sopra (operatori aritmetici, `zero()`, funzioni di distanza). Vedi `world/point.rs` e `world/position.rs` per l'elenco completo dei metodi — sono per lo più usati internamente e tramite i campi `get_position()` / `WasmPoint` su `WasmNode`/`WasmWire`.
