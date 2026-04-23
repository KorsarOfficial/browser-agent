use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PressKeyParams {
    #[schemars(description = "Key to press (e.g. Enter, Tab, Escape, ArrowDown, Backspace)")]
    pub key: String,
}
