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
    datatype: String,
    properties: Option<Box<StructuredOutput>>,
    items: Option<Box<StructuredOutput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonSchema {
    name: String,
    strict: bool,
    schema: StructuredOutput,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    response_type: String,
    schema: JsonSchema,
}
