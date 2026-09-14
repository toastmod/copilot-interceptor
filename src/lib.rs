pub mod http_client;
pub mod openai_client;
pub mod openai_server;
pub mod inner;
pub mod intercept;
pub mod prelude;

mod test {
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use warp::filters::sse::Event;

    use crate::{ inner::start_server, intercept::Interceptor, openai_server::OpenAiService };

    struct CustomService;
    impl Interceptor for CustomService {
        fn make_client_request_streaming(
            request_body: crate::prelude::openai_server::OpenAiRequest,
            headers: reqwest::header::HeaderMap<reqwest::header::HeaderValue>
        ) -> tokio_stream::wrappers::ReceiverStream<
            Result<warp::filters::sse::Event, std::convert::Infallible>
        > {
            let (tx, rx) = mpsc::channel(1);
            tokio::spawn(async move {
                tx.send(Ok(Event::default().data(include_str!("./test.json")))).await;
                tx.send(Ok(Event::default().data("[DONE]"))).await;
            });
            ReceiverStream::new(rx)
        }
    }

    #[tokio::test]
    async fn simple_interception() {
        start_server(([0, 0, 0, 0], 10001), OpenAiService {}).await;
    }

    #[tokio::test]
    async fn custom_interception() {
        start_server(([0, 0, 0, 0], 10001), CustomService {}).await;
    }
}
