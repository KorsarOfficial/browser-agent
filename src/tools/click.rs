use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickParams {
    #[schemars(description = "CSS selector of element to click")]
    pub selector: String,
}
