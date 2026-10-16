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
| `get_wire_node_render_x/y(wire_idx, node_idx) -> f32` | Dove il nodo viene disegnato dall'ultimo `build_render_data` (interpolato) |
| `get_wire_node_fixed(wire_idx, node_idx) -> bool` | Se un nodo è fisso, senza copiarlo |
| `get_wire_node_x/y(wire_idx, node_idx) -> f32` | Coordinate del nodo |
| `get_wire_node_count(wire_idx) -> usize` | Numero di nodi di un cavo |
| `add_wire_node(wire_idx, x, y, fixed)` | Aggiunge un nodo in coda |
| `add_wire_node_at(wire_idx, node_idx, x, y, fixed)` | Inserisce un nodo a un indice |
| `remove_wire_node(wire_idx, node_idx) -> bool` | Rimuove un nodo |
| `set_wire_node_count(wire_idx, node_count)` | Ridimensiona un cavo, ridistribuendo i nodi uniformemente |
| `set_wire_node_fixed(wire_idx, node_idx, fixed)` | Fissa/libera un nodo. Liberare un nodo fisso lo lancia con la velocità del suo ultimo spostamento (vedi [Spostare i nodi](#spostare-i-nodi-dallesterno)) |
| `set_wire_node_position(wire_idx, node_idx, x, y)` | Sposta un nodo dall'esterno della simulazione (vedi [Spostare i nodi](#spostare-i-nodi-dallesterno)) |
| `set_wire_start(wire_idx, x, y)` / `set_wire_end(wire_idx, x, y)` | Sposta gli estremi fissi |
| `get_wire_radius(wire_idx) -> f32` / `set_wire_radius(wire_idx, radius)` | Spessore del cavo (non cambia la distanza tra i nodi) |
| `get_wire_link_distance(wire_idx) -> f32` / `set_wire_link_distance(wire_idx, distance)` | Distanza a riposo tra nodi vicini |
| `get_wire_render_type(wire_idx) -> u8` / `set_wire_render_type(wire_idx, render_type)` | Tipo di rendering di un cavo (`0` = segmenti, `1` = bezier), usato dal prossimo `build_render_data` |
| `is_wire_sleeping(wire_idx) -> bool` | Se un cavo dorme (vedi [Sospensione](#sospensione)) |

**Simulazione**
| Metodo | Descrizione |
|---|---|
| `update()` | `step()` seguito da `build_render_data(1.0)` |
| `step()` | Avanza la fisica di tutti i cavi di un passo (`substeps` × `time_step`); non tocca il buffer di rendering |
| `build_render_data(alpha)` | Ricostruisce il buffer di rendering disegnando ogni nodo mobile ad `alpha` (0..1) del tragitto tra la sua posizione prima dell'ultimo passo e quella attuale; i nodi fissi sono disegnati dove sono. Va chiamato a ogni frame, anche quando non è stato eseguito nessun passo |
| `set_mouse(x, y)` | Imposta la posizione del puntatore per le collisioni |
| `set_held_wire(wire_idx)` / `release_held_wire()` / `get_held_wire() -> Option<usize>` | Il cavo tenuto dall'utente (che ne trascina un capo): vedi [Cavo tenuto](#cavo-tenuto) |
| `sleeping_wire_count() -> usize` / `wake_all()` | Cavi addormentati, e risveglio di tutti |
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
| `pointer_radius` | `20.0` | Raggio entro cui il punto tenuto respinge gli altri cavi mentre un cavo è tenuto |
| `response_coef` | `0.0` | Intensità della risposta alla self-collision |
| `friction` | `0.95` | Fattore di smorzamento Verlet |
| `acceleration` | `(0.0, 10.0)` | Accelerazione globale (gravità), disponibili anche `get_acceleration_x/y()` |
| `time_step` | `0.3` | Tempo simulato per sotto-passo. Più grande è più veloce ma meno stabile |
| `substeps` | `2` | Sotto-passi per `step()` (almeno 1). Ognuno avanza il tempo di `time_step`, quindi cambia la velocità oltre alla precisione |
| `constraint_iterations` | `1` | Passate di rilassamento dei vincoli per sotto-passo (almeno 1): di più rendono i cavi più rigidi, senza cambiare la velocità |
| `render_alpha` | `1.0` | Sola lettura: l'`alpha` dell'ultimo `build_render_data` |
| `sleep_enabled` | `true` | Addormenta i cavi fermi (disattivarlo li risveglia tutti) |
| `sleep_threshold` | `0.01` | Spostamento massimo di un nodo in un passo che conta come fermo |
| `sleep_steps` | `30` | Passi fermi consecutivi prima che un cavo si addormenti (almeno 1) |

**Passi indipendenti dal frame rate.** Il passo ha un delta di tempo fisso: chiama `step()` a frequenza fissa (es. 60 al secondo, con un accumulatore) e `build_render_data(accumulatore / durata_passo)` a ogni frame. La fisica si comporta allo stesso modo su ogni schermo, e uno schermo più veloce della frequenza dei passi disegna comunque una posizione nuova a ogni frame.

## `WasmWire`

Costruito tramite `WasmWire::new()` (cavo di default a 30 nodi) o le factory statiche `new_wire(...)` / `new_with_count(...)` (rispecchiano `WasmWorld::add_wire[_with_count]`). Le factory dispongono i nodi in modo uniforme sulla retta tra i due estremi (fissi), così un cavo nuovo parte da dove viene disegnato invece di arrivare in volo da un punto fisso.

| Metodo | Descrizione |
|---|---|
| `node_count() -> usize` | Numero di nodi |
| `get_node(index) -> Option<WasmNode>` | Copia di un nodo |
| `get_node_x/y(index) -> f32` | Coordinate del nodo |
| `get_radius() -> f32` / `set_radius(radius)` | Solo lo spessore |
| `get_link_distance() -> f32` / `set_link_distance(distance)` | Distanza a riposo tra nodi vicini (`link_target_distance`) |
| `add_node(x, y, fixed)` / `add_node_at(index, x, y, fixed)` / `remove_node(index) -> bool` | Modifiche strutturali; `remove_node` rifiuta di scendere sotto 2 nodi |
| `get_render_type() -> u8` / `set_render_type(render_type)` | `0` = segmenti, `1` = bezier |
| `set_node_count(new_count)` | Ricostruisce la lista dei nodi, mantenendo fissi gli estremi, distribuendo uniformemente il resto |
| `get_start()` / `get_end() -> WasmPoint` | Posizioni degli estremi |
| `set_start(x, y)` / `set_end(x, y)` | Sposta gli estremi |
| `set_node_fixed(node_idx, fixed)` / `set_node_position(node_idx, x, y)` | Modifiche per singolo nodo; `set_node_position` posiziona il nodo come `WasmNode::move_to` |
| `is_node_fixed(index) -> bool` | Se un nodo è fisso |
| `is_sleeping() -> bool` | Se il cavo dorme |
| `length() -> f32` | Somma delle distanze dei segmenti (lunghezza della polilinea) |
| `check_collision(point, pointer_radius)` | Spinge i nodi mobili fuori da `radius + pointer_radius` rispetto a `point` |
| `update(dt, friction, acceleration)` | Integra i nodi, poi rilassa una volta il vincolo di distanza tra vicini |
| `update_with_iterations(dt, friction, acceleration, iterations)` | Come sopra, rilassando il vincolo `iterations` volte |
| `is_node_fixed(index) -> bool` | Se un nodo è fisso |
| `check_wire_elements_collisions(response_coef)` | Self-collision |
| `check_mouse_collision(mouse, mouse_radius)` | Repulsione dal puntatore |
| `invalidate()` | Risveglia il cavo e azzera il conteggio dei passi fermi |
| `optimal_length(start, end, node_radius) -> usize` (statico) | Euristica sul numero di nodi usata da `WasmWorld::wire_optimal_length` |

Campi pubblici: `radius: f32`, `link_target_distance: f32`, `render_type: u8`.

### Cavo tenuto

Mentre l'utente trascina un capo di un cavo, chi usa il motore chiama `set_held_wire(indice)` (e `release_held_wire()` al rilascio). Il mouse, che viene tenuto sul capo trascinato:
- non respinge per niente il cavo tenuto, altrimenti allontanerebbe i nodi di quel cavo dal capo trascinato e lo stirerebbe;
- respinge gli altri cavi entro `pointer_radius` invece di `mouse_radius`.

Senza cavi tenuti, il mouse respinge tutti i cavi entro `mouse_radius`. L'indice del cavo tenuto segue il cavo quando ne viene cancellato uno precedente, e si azzera quando viene cancellato il cavo tenuto stesso.

### Sospensione

Un cavo i cui nodi si sono spostati tutti meno di `sleep_threshold` in ciascuno di `sleep_steps` passi consecutivi si addormenta: `step()` lo salta e la sua velocità residua viene azzerata. Si risveglia appena qualcosa può rimetterlo in movimento: il mouse arriva vicino al suo riquadro, un nodo viene spostato, fissato o rilasciato, vengono aggiunti o tolti nodi, cambia il raggio o la distanza tra i nodi, o cambia un parametro globale (gravità, attrito, coefficiente di risposta, time step, sotto-passi, iterazioni dei vincoli). I cavi addormentati vengono comunque disegnati.

### Spostare i nodi dall'esterno

`set_wire_node_position` / `set_wire_start` / `set_wire_end` (e `WasmNode::move_to`) posizionano un nodo dall'esterno della simulazione:
- un nodo **libero** viene teletrasportato, senza velocità;
- un nodo **fisso** (un capo trascinato o collegato) cambia solo la posizione corrente, e la posizione precedente lo segue a ogni integrazione, così non accumula come velocità tutta la strada fatta mentre era fisso. Quando viene rilasciato con `set_wire_node_fixed(…, false)` viene lanciato con lo spostamento dell'ultimo passo, diviso tra i sotto-passi; un nodo tenuto fermo cade e basta.

Collisioni e vincoli dentro il motore continuano a usare `WasmNode::set_position`, che sposta solo la posizione corrente, così le loro correzioni trasmettono quantità di moto (Verlet).

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
| `update_position(dt, friction, acceleration)` | Passo di integrazione Verlet smorzata (per un nodo fisso allinea solo la posizione precedente a quella corrente) |
| `set_position(x, y)` | Imposta solo la posizione corrente (lo spostamento conta come velocità): la usano collisioni e vincoli |
| `move_to(x, y)` | Posiziona il nodo dall'esterno della simulazione (vedi [Spostare i nodi](#spostare-i-nodi-dallesterno)) |
| `update_with_acceleration(acceleration, dt, friction)` | Punto di ingresso alternativo per l'integrazione tramite `WasmPosition::integrate` |
| `translate(dx, dy)` | Delta di posizione diretto |
| `reset_velocity()` | Azzera velocità e storico Verlet |
| `distance_to(other)` / `distance_squared_to(other) -> f32` | Funzioni di distanza |
| `print() -> String` | Stringa di debug (anche via `Display`) |

Campi pubblici: `position: WasmPosition`, `velocity: WasmPoint`, `fixed: bool`.

## `WasmPoint` / `WasmPosition`

Tipi vettore/posizione di basso livello usati in tutta l'API sopra (operatori aritmetici, `zero()`, funzioni di distanza). Vedi `world/point.rs` e `world/position.rs` per l'elenco completo dei metodi — sono per lo più usati internamente e tramite i campi `get_position()` / `WasmPoint` su `WasmNode`/`WasmWire`.
