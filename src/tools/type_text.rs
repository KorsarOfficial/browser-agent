use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TypeParams {
    #[schemars(description = "CSS selector of input element")]
    pub selector: String,
    #[schemars(description = "Text to type into the element")]
    pub text: String,
}
