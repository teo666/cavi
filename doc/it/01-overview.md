# Cavi — Panoramica

`cavi` è una libreria Rust compilata in **WebAssembly** (tramite `wasm-bindgen` / `wasm-pack`) che implementa una simulazione fisica 2D di cavi flessibili ("wires"). È pensata per essere usata dal browser dal progetto TypeScript gemello [`cavijs`](../../../cavijs/doc/it/01-overview.md), che si occupa del rendering e degli aspetti DOM/UI.

## Scopo

Il motore simula cavi appesi/trascinabili composti da punti collegati ("nodi"), usando uno schema di **integrazione di Verlet** con:

- Gravità / accelerazione configurabile
- Attrito (smorzamento)
- Vincoli di distanza tra nodi vicini (mantiene i segmenti del cavo vicini a una lunghezza target)
- Collisione tra i nodi dello stesso cavo (self-collision)
- Collisione/repulsione rispetto al puntatore del mouse

La simulazione produce soltanto **dati numerici** (posizioni dei nodi, punti di controllo delle curve). Deliberatamente **non** conosce nulla di rendering, colori o DOM: questa responsabilità è lasciata interamente al lato JavaScript/TypeScript, in modo che il renderer possa essere sostituito o esteso liberamente.

## Perché WebAssembly

L'integrazione fisica e i controlli di collisione girano ad ogni frame e scalano con il numero di cavi/nodi; farlo in Rust/WASM mantiene le prestazioni elevate ed evita allocazioni nei percorsi critici dove possibile (vedi `wire_data_buffer` in `WasmWorld`, riutilizzato tra i frame ed esposto a JS come vista `Float32Array` a copia zero).

## Stato del progetto

Questo repository è stato avviato a partire dal template [`rustwasm/wasm-pack-template`](https://github.com/rustwasm/wasm-pack-template) — il `README.md` nella root contiene ancora le istruzioni generiche del template invece della documentazione specifica del progetto; questa cartella `doc/` è la documentazione effettiva del progetto.

## Relazione con `cavijs`

- `cavi` (questo progetto): nucleo della simulazione fisica, compilato in un pacchetto WASM, pubblicato/consumato come pacchetto npm `cavi`.
- `cavijs`: wrapper TypeScript, renderer su canvas e Web Components (`Cavi`, `World`, `Wire`, `Node`, `Renderer`, `cavi-jack`, `cavi-plug`) costruiti sopra di esso.

Vedi anche:
- [Architettura](./02-architecture.md)
- [Riferimento API](./03-api.md)
- [Sviluppo / build](./04-development.md)
