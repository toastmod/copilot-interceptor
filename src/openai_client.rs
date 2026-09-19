use std::collections::HashMap;

use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StructuredOutput {
    #[serde(rename = "type")]
    pub datatype: String,
    pub properties: Option<Box<StructuredOutput>>,
    pub items: Option<Box<StructuredOutput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonSchema {
    pub name: String,
    pub strict: bool,
    pub schema: StructuredOutput,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_type: String,
    pub schema: JsonSchema,
}
