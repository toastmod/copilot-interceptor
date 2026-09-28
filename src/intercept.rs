use std::{ convert::Infallible, sync::Arc };

use eventsource_stream::Eventsource;
use reqwest::header::{ HeaderMap, HeaderValue };
use tokio::sync::mpsc;
use tokio_stream::{ StreamExt, wrappers::ReceiverStream };

use crate::openai_server::{ OpenAiRequest };

pub trait Interceptor {
    fn make_client_request_streaming(
        interceptor: Arc<Self>,
        request_body: OpenAiRequest,
        headers: HeaderMap<HeaderValue>
    ) -> ReceiverStream<Result<warp::filters::sse::Event, Infallible>> {
        let (tx, rx) = mpsc::channel(1);
        let client = reqwest::Client::new();
        println!("Requesting llama.cpp");
        let map = headers.iter().filter_map(|x| {
            // Filter in any HTTP headers here...
            if
                [
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
                ].contains(&x.0.as_str())
            {
                Some((x.0.clone(), x.1.clone()))
            } else {
                None
            }
        });
        let headers = HeaderMap::from_iter(map);

        tokio::spawn(async move {
            match
                client
                    .post("http://desktop-ttjki31:10000/v1/chat/completions")
                    .headers(headers)
                    .header("Connection", "keep-alive")
                    .json(&request_body)
                    .send().await
            {
                Ok(response) => {
                    let mut stream = response.bytes_stream().eventsource();
                    while let Some(x) = stream.next().await {
                        println!("{:#?}", x);
                        let event = if let Ok(xx) = x {
                            Ok(
                                warp::filters::sse::Event
                                    ::default()
                                    .data(xx.data)
                                    .id(xx.id)
                                    .event(xx.event)
                            )
                        } else {
                            Ok(warp::filters::sse::Event::default().data("An error occured."))
                        };
                        if tx.send(event).await.is_err() {
                            // Receiver dropped, so we can stop.
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("Error sending request to llama.cpp: {:?}", e);
                    let event = warp::filters::sse::Event
                        ::default()
                        .data(format!("Error connecting to backend: {}", e));
                    let _ = tx.send(Ok(event)).await;
                }
            }
        });

        ReceiverStream::new(rx)
    }
}
