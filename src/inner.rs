use tokio;
use warp::{ Filter, sse::Event };
use std::{ sync::Arc, convert::Infallible };

use crate::openai_client::make_client_request_streaming;
use crate::openai_server::{ OpenAiRequest, OpenAiService };
use tokio_stream::wrappers::ReceiverStream;
use warp::{ filters::method::head };
use warp::http::HeaderMap;

#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

// #[tokio::main]
pub async fn start_server<T>(addr: ([u8; 4], u16)) -> Result<(), Box<dyn std::error::Error>> {
    let service = Arc::new(OpenAiService {});

    let service_provider = warp::any().map(move || Arc::clone(&service));

    let chat_route = warp
        ::path("chat")
        .and(warp::path("completions"))
        .and(warp::post())
        .and(warp::header::headers_cloned())
        .and(warp::body::json())
        .and(service_provider.clone())
        .and_then(
            |
                headers: HeaderMap,
                request_body: OpenAiRequest,
                service: Arc<OpenAiService>
            | async move {
                println!("Incoming Body: {:?}\n\n", request_body);
                let events = make_client_request_streaming::<T>(request_body, headers).await;
                Ok::<_, warp::Rejection>(warp::sse::reply(warp::sse::keep_alive().stream(events)))
            }
        );

    let list_models_route = warp
        ::path("models")
        .and(warp::get())
        .and(service_provider.clone())
        .and_then(|service: Arc<OpenAiService>| async move {
            println!("Models!");
            let service = Arc::clone(&service);
            match service.list_models().await {
                Ok(models) => Ok(warp::reply::json(&models)),
                Err(e) =>
                    Err(warp::reject::custom(CustomError(format!("Failed to list models: {}", e)))),
            }
        });

    let get_status_route = warp
        ::path("status")
        .and(warp::get())
        .and(service_provider)
        .and_then(|service: Arc<OpenAiService>| async move {
            let service = Arc::clone(&service);
            match service.get_status().await {
                Ok(status) => {
                    println!("Status!");
                    Ok(
                        warp::reply::with_status(
                            warp::reply::json(&status),
                            warp::http::StatusCode::OK
                        )
                    )
                }
                Err(e) =>
                    Err(warp::reject::custom(CustomError(format!("Failed to get status: {}", e)))),
            }
        });

    // 4. Run the server
    println!(
        "Server running on http://{}:{}",
        addr.0
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("."),
        addr.1
    );
    warp
        ::serve(warp::path("v1").and(chat_route.or(list_models_route).or(get_status_route)))
        .run(addr).await;

    Ok(())
}
