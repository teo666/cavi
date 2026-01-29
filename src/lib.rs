mod utils;
mod world;

#[cfg(test)]
mod point_tests;
#[cfg(test)]
mod node_tests;
#[cfg(test)]
mod wire_tests;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}