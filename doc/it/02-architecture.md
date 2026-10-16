# Cavi — Architettura

## Struttura dei moduli

```
src/
├── lib.rs              punto di ingresso, collega i moduli, extern wasm-bindgen (alert)
├── utils.rs             utilità varie (setup panic hook, ecc.)
└── world.rs              ri-esporta i sottomoduli di world
    world/
    ├── point.rs           WasmPoint — vettore 2D (x, y) con operazioni vettoriali
    ├── position.rs        WasmPosition — stato di posizione per Verlet (curr / old)
    ├── node.rs            WasmNode — un singolo punto materiale in un cavo
    └── wire.rs            WasmWire — una catena di nodi che rappresenta un cavo
```

`WasmWorld` risiede direttamente in `world.rs` ed è il contenitore principale della simulazione.

## Tipi principali

### `WasmPoint` (`world/point.rs`)
Un tipo vettore 2D minimale (`x: f32`, `y: f32`) con operatori aritmetici, usato ovunque siano rappresentate posizioni/velocità/forze.

### `WasmPosition` (`world/position.rs`)
Incapsula lo stato di **integrazione di Verlet** di un nodo: posizione corrente (`curr`) e posizione precedente (`old`). La velocità è implicita (`curr - old`), il trucco standard usato dagli integratori di Verlet per evitare di memorizzare la velocità esplicitamente pur supportando attrito/smorzamento.

### `WasmNode` (`world/node.rs`)
Un singolo punto materiale in un cavo:
- `position: WasmPosition`
- `velocity: WasmPoint` (velocità esplicita, usata da alcuni metodi ausiliari basati su forze, oltre alla posizione Verlet)
- `fixed: bool` — i nodi fissati (es. gli estremi del cavo) non si muovono mai durante l'integrazione

Il movimento avanza con `update_position(dt, friction, acceleration)`, che applica l'integrazione di Verlet smorzata più l'accelerazione (gravità).

### `WasmWire` (`world/wire.rs`)
Una lista ordinata di `WasmNode` più parametri a livello di cavo:
- `radius` — raggio visivo/di collisione del cavo
- `link_target_distance` — la distanza che i nodi vicini cercano di mantenere (un vincolo di distanza, rilassato ad ogni passo fisico)
- `render_type` — `0` = segmenti retti, `1` = curva smussata (Bezier, vedi sotto)

Responsabilità a livello di cavo:
- `update(dt, friction, acceleration)` — integra tutti i nodi, poi rilassa il vincolo di distanza tra ogni coppia di nodi vicini
- `check_wire_elements_collisions(response_coef)` — allontana i nodi dello stesso cavo se si avvicinano più del diametro del cavo (self-collision)
- `check_mouse_collision(mouse, mouse_radius)` — allontana i nodi dal puntatore del mouse
- Modifiche strutturali: `add_node`, `add_node_at`, `remove_node`, `set_node_count` (ridistribuisce i nodi in modo uniforme tra gli estremi fissi)

### `WasmWorld` (`world.rs`)
Il contenitore della simulazione e il principale punto di ingresso esposto a JS:
- Possiede tutti i `WasmWire`, la posizione del mouse, il cavo tenuto e i parametri globali (`mouse_radius`, `pointer_radius`, `response_coef`, `friction`, `acceleration`, impostazioni del passo e della sospensione).
- `step()` esegue il ciclo fisico per **tutti** i cavi (`substeps` sotto-passi, default 2: gestione collisioni, integrazione, rilassamento vincoli). `build_render_data(alpha)` ricalcola il buffer di punti pronti per il rendering per ogni cavo, interpolato tra gli ultimi due passi. `update()` fa entrambe le cose, con `alpha = 1`.
- Possiede `wire_data_buffer: Vec<f32>` — un buffer piatto ricostruito ad ogni chiamata di `build_render_data` ed esposto a JS tramite `wire_data_ptr()` / `wire_data_len()`, così il renderer può leggere direttamente la memoria lineare WASM (**a copia zero**) invece di attraversare il confine JS/WASM per ogni nodo.

## Passo fisico, per ogni chiamata a `step()`

1. Ogni cavo memorizza le posizioni dei suoi nodi (`snapshot_positions`), per l'interpolazione del rendering.
2. Per `substeps` sotto-passi (default 2):
   a. salta i cavi addormentati (risvegliandone uno se il mouse è a portata), poi `check_mouse_collision` — respinge i nodi vicini al mouse: entro `mouse_radius`, oppure entro `pointer_radius` e mai sul cavo tenuto mentre un cavo è tenuto
   b. `check_wire_elements_collisions` — respinge i nodi di un cavo tra loro (self-collision)
   c. `update_with_iterations(time_step, friction, acceleration, constraint_iterations)` — integra le posizioni con Verlet (`time_step` di default 0.3), poi rilassa il vincolo di distanza tra vicini (di default una volta)

## Dati di rendering, per ogni chiamata a `build_render_data(alpha)`

Ricostruisce `wire_data_buffer` per **ogni** cavo, convertendo le posizioni dei nodi in punti pronti per il rendering secondo `render_type`. Ogni nodo mobile viene disegnato in `prev + (curr - prev) * alpha`, dove `prev` è la sua posizione prima dell'ultimo passo. I nodi fissi, e tutti i nodi di un cavo il cui numero di nodi è cambiato dall'ultimo passo, vengono disegnati dove sono ora: un nodo fisso è posizionato dall'app (un capo collegato o trascinato) e non deve restare indietro.

Il passo ha un delta di tempo fisso, e con l'integrazione di Verlet la velocità è implicita (`curr - old`) e vale solo con un passo costante; anche l'attrito è applicato per passo. Quindi chi usa il motore non deve far variare la durata del passo con il frame rate: deve chiamare `step()` a frequenza fissa e `build_render_data(avanzo / durata_passo)` a ogni frame. Il disegno resta indietro al massimo di un passo, e uno schermo più veloce della frequenza dei passi riceve una posizione nuova a ogni frame.

## Generazione delle curve

Due strategie di rendering, selezionate per-cavo tramite `render_type`:

- **Segmenti (`0`)** — `nodes_to_segments`: le posizioni dei nodi sono usate così come sono (polilinea).
- **Bezier (`1`, default)** — `catmull_to_bezier_points`: converte la spline Catmull-Rom implicita nei nodi consecutivi in una sequenza di punti di controllo Bezier cubici, producendo una curva smussata che passa per tutti i nodi. È la conversione standard Catmull-Rom → Bezier (i due punti di controllo di ogni segmento interno sono derivati dalle posizioni dei nodi vicini).

## Formato del buffer dati dei cavi

Per ogni cavo, viene aggiunto a `wire_data_buffer` quanto segue (tutti `f32`):

```
[node_count, radius, render_type, path_length, ...path_data]
```

Dove `path_data` è una lista piatta di punti `(x, y)` per i segmenti (`render_type == 0`) oppure una lista piatta di tuple Bezier `(cp1x, cp1y, cp2x, cp2y, x, y)` (`render_type == 1`). `path_length` è il numero di valori `f32` in `path_data` (cioè `points.len() * 2`).

Questo buffer è pensato per essere letto lato JS come un'unica vista `Float32Array` sulla memoria WASM (vedi `Renderer.drawAllWires` in `cavijs`), evitando chiamate FFI per ogni nodo.

## Modello di collisioni e vincoli

- **Vincolo di distanza** (`WasmWire::update`): per ogni coppia di nodi vicini, sposta entrambi verso/da l'altro di metà del delta tra la distanza reale e `link_target_distance`, a meno che un nodo sia `fixed`.
- **Self-collision** (`check_wire_elements_collisions`): se due nodi vicini sono più vicini del diametro del cavo, li allontana, scalato da `response_coef` (default `0.0`, cioè disabilitato, nel costruttore di `World` in `cavijs`).
- **Collisione col mouse** (`check_mouse_collision` / `check_collision`): spinge i nodi mobili fuori da un cerchio di raggio `mouse_radius + wire.radius` attorno al mouse/puntatore (`pointer_radius` mentre un cavo è tenuto). `WasmWire::intersect()` confronta prima il punto con il riquadro del cavo allargato di quel raggio, e i nodi vengono confrontati per distanza al quadrato, così i cavi e i nodi lontani non costano nessuna radice quadrata.

Dopo i sotto-passi, ogni cavo sveglio conta il passo come fermo o no (`note_step`) e si addormenta dopo `sleep_steps` passi fermi — vedi [API → Sospensione](./03-api.md#sospensione).

## Test

I test unitari sono collocati accanto a `lib.rs` e attivati con `#[cfg(test)]`:
- `point_tests.rs` — matematica vettoriale di `WasmPoint`
- `node_tests.rs` — comportamento di integrazione/stato di `WasmNode`
- `wire_tests.rs` — comportamento di vincoli/collisioni di `WasmWire`
- `world_tests.rs` — passi di `WasmWorld`, interpolazione del rendering e configurazione

I test di integrazione basati su browser si trovano in `tests/web.rs` (eseguiti con `wasm-pack test --headless --firefox`).
