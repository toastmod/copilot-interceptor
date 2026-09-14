use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: String,
}
