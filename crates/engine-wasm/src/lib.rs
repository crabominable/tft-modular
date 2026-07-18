// crates/engine-wasm/src/lib.rs
//! Browser glue: JSON string bridge over `engine_core::Match`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn wasm_engine_version() -> String {
    engine_core::engine_version().to_string()
}

/// WASM-facing match handle. Plugin and commands cross the boundary as JSON strings.
#[wasm_bindgen]
pub struct WasmMatch {
    inner: engine_core::Match,
}

#[wasm_bindgen]
impl WasmMatch {
    /// Create a match from a plugin bundle JSON and seed.
    ///
    /// Expected plugin shape:
    /// `{ "manifest": {...}, "units": [...], "traits": [...], "abilities": [...] }`
    #[wasm_bindgen(constructor)]
    pub fn new(plugin_json: &str, seed: u64) -> Result<WasmMatch, JsValue> {
        let plugin: engine_core::PluginData = serde_json::from_str(plugin_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            inner: engine_core::Match::new(plugin, seed),
        })
    }

    /// Apply a player command (JSON). Returns serialized `Vec<Event>`.
    pub fn apply(&mut self, player_id: u8, command_json: &str) -> Result<String, JsValue> {
        let cmd: engine_core::Command = serde_json::from_str(command_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let events = self
            .inner
            .apply(player_id, cmd)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&events).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Full match snapshot as JSON (`MatchSnapshot`).
    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.snapshot())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Deterministic state hash as 16-char lowercase hex.
    pub fn state_hash(&self) -> String {
        format!("{:016x}", self.inner.state_hash())
    }
}
