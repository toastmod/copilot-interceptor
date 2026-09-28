# Copilot Interceptor
This is an experimental tool for building interceptors for VSCode Copilot streams to `llama.cpp`.\
It can be used to make debuggers or add complex logic to local agents.

## Quick Example
```rust
use copilot_interceptor::prelude::*;

// A simple interception service that prints the stream.
// Good for debugging.
struct InterceptorService;
impl Interceptor for InterceptorService {
  // Enter your Ollama/llama.cpp server URL like so
  const LLAMA_HOST_URL: &'static str = "http://localhost:11434";
}

#[tokio::main]
async fn main() { 
    let server = start_server(([0, 0, 0, 0], 10001), InterceptorService {});
}

```

## Custom Interceptor Example
```rust
use std::sync::Arc;
use copilot_interceptor::prelude::{ tokio::sync::mpsc, tokio_stream::wrappers::ReceiverStream, * };

// Implement a custom interception service.
struct CustomService;
impl Interceptor for CustomService {

  // Enter your Ollama/llama.cpp server URL like so
  const LLAMA_HOST_URL: &'static str = "http://localhost:11434";

  // The following code is the default implementation. 
  fn make_client_request_streaming(
    // You will receive an Arc of your service, so consider atomics or mpsc channels. 
    service: Arc<Self>,
    request_body: OpenAiRequest,
    headers: HeaderMap<HeaderValue>,
  ) -> ReceiverStream<Result<Event, Infallible>> {

    // Create an mpsc channel to bridge the incoming and outgoing streams.
    let (tx, rx) = mpsc::channel(1);

    let client = reqwest::Client::new();

    // This is optional, but you can also manage HTTP header data.
    // This map whitelists headers from the original request to pass through the intercept.
    let map = headers.iter().filter_map(|x| {
      // Filter in any HTTP headers here...
      if [
        "openai-intent",
        "user-agent",
        "x-agent-task-id",
        "x-github-api-version",
        "x-initiator",
        "x-interaction-id",
        "x-interaction-type",
        "x-onbehalf-extension-id",
        "x-request-id",
        "x-vscode-user-agent-library-version",
        "accept-encoding",
        "accept",
        "connection",
      ]
      .contains(&x.0.as_str())
      {
        Some((x.0.clone(), x.1.clone()))
      } else {
        None
      }
    });
    let headers = HeaderMap::from_iter(map);

    // This interceptor passes the request to llama.cpp and streams back the result.
    println!("Requesting llama.cpp");
    tokio::spawn(async move {
      // Make a request to your local llama.cpp server.
      // Make sure to use the `v1/chat/completions` route. (other APIs aren't supported yet)
      match client

        .post(format!("{}/v1/chat/completions", Self::LLAMA_HOST_URL))
        .headers(headers)
        .header("Connection", "keep-alive")
        .json(&request_body)
        .send()
        .await
      {
        Ok(response) => {
          let mut stream = response.bytes_stream().eventsource();
          while let Some(x) = stream.next().await {
            println!("{:?}", x);
            let event = if let Ok(xx) = x {
              // TODO: Deserialize xx.data as an OpenAiResponse type 
              Ok(
                Event::default()
                  .data(xx.data)
                  .id(xx.id)
                  .event(xx.event),
              )
            } else {
              Ok(Event::default().data("An error occured."))
            };
            if tx.send(event).await.is_err() {
              // Receiver dropped, so we can stop.
              break;
            }
          }
        }
        Err(e) => {
          println!("Error sending request to llama.cpp: {:?}", e);
          let event = Event::default()
            .data(format!("Error connecting to backend: {}", e));
          let _ = tx.send(Ok(event)).await;
        }
      }
    });

    // Your output stream items should be `warp::filter::sse::Event` type.
    // The "data" value should be an `OpenAiResponse` serialized into a JSON string.
    ReceiverStream::new(rx)
  }
}

#[tokio::main]
async fn main() {
    
    let server = start_server(([0, 0, 0, 0], 10001), CustomService {});
}

```
