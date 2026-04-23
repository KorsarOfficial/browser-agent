use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NavParams {
    #[schemars(description = "URL to navigate to")]
    pub url: String,
}
