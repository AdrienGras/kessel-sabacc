use wasm_bindgen::prelude::*;

/// Version string — smoke test to validate WASM loading.
#[wasm_bindgen]
pub fn version() -> String {
    format!("sabacc-wasm v{}", env!("CARGO_PKG_VERSION"))
}

/// Ping -> "pong" — minimal integration test.
#[wasm_bindgen]
pub fn ping() -> String {
    "pong".into()
}
