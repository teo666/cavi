mod utils;
mod world;

#[cfg(test)]
mod point_tests;
#[cfg(test)]
mod node_tests;
#[cfg(test)]
mod wire_tests;
#[cfg(test)]
mod world_tests;

use wasm_bindgen::prelude::*;

/// Runs once when the WASM module is instantiated: a panic is then logged
/// with its message and location to `console.error`, instead of the bare
/// "unreachable executed" the browser reports otherwise.
#[wasm_bindgen(start)]
fn start() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}