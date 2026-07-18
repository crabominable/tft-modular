// crates/engine-wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn wasm_engine_version() -> String {
    engine_core::engine_version().to_string()
}
