use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickAtParams {
    #[schemars(description = "X coordinate in pixels from top-left")]
    pub x: f64,
    #[schemars(description = "Y coordinate in pixels from top-left")]
    pub y: f64,
}
