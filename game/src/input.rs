use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Input {
    Jump,
    Duck,
    Unduck,
    None,
}
