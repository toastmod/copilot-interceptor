use std::{ convert::Infallible, sync::Arc };

use reqwest::header::{ HeaderMap, HeaderValue };
use tokio_stream::wrappers::ReceiverStream;

use crate::openai_server::{ OpenAiRequest };

pub trait Interceptor {
    fn make_client_request_streaming(
        interceptor: Arc<Self>,
        request_body: OpenAiRequest,
        headers: HeaderMap<HeaderValue>
    ) -> ReceiverStream<Result<warp::filters::sse::Event, Infallible>>;
}
