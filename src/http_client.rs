use reqwest;
use tokio;
use serde::{ Serialize, Deserialize };

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

pub async fn make_request(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Making request to http://localhost:10000/v1/chat/completions...");

    let request_body = ChatCompletionRequest {
        model: "gpt-6-astra".to_string(), // Using the model from the example
        messages: vec![
            Message {
                role: "developer".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            Message { role: "user".to_string(), content: prompt.to_string() }
        ],
        temperature: 0.7,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:10000/v1/chat/completions")
        .json(&request_body)
        .send().await?;

    let status = response.status();
    println!("Request successful. Status: {}", status);

    // Read the entire response body as text first for better debugging
    let response_text = response.text().await?;

    // Attempt to parse the response to extract content
    match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(json_value) => {
            // Extract content from the response structure as per the example
            let response_text = json_value
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .unwrap_or("Error: Could not find 'content' in response structure");

            println!("Response body: {}", response_text);
            Ok(response_text.to_string())
        }
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
            Err(format!("Failed to parse server response: {}", e).into())
        }
    }
}
