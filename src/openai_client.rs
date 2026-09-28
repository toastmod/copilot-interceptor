use std::collections::HashMap;

use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonSchema {
    pub name: String,
    pub strict: bool,
    pub schema: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_type: String,
    pub json_schema: JsonSchema,
}

#[macro_export]
macro_rules! json_response_format {
    ($body:tt) => {
        serde_json
            ::to_value(ResponseFormat {
                response_type: "json_schema".to_string(),
                json_schema: JsonSchema $body,
            })
            .unwrap()
    };
}