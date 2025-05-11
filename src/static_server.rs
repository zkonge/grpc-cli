use std::{
    convert::Infallible,
    future::{Ready, ready},
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{
    StreamExt,
    future::BoxFuture,
    stream::{self, BoxStream, pending},
};
use http::{Request, Response, Uri, header::CONTENT_TYPE};
use prost_reflect::{DynamicMessage, MethodDescriptor};
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tonic::{
    Status,
    body::Body as TonicBody,
    metadata::GRPC_CONTENT_TYPE,
    server::{Grpc, ServerStreamingService, UnaryService},
};
use tower_service::Service;

use crate::codec::DynamicProstCodec;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type BoxResultFuture<T, E> = BoxFuture<'static, Result<T, E>>;

struct InnerUnaryService {
    resp_body: DynamicMessage,
}

impl UnaryService<DynamicMessage> for InnerUnaryService {
    type Response = DynamicMessage;
    type Future = Ready<Result<tonic::Response<Self::Response>, Status>>;

    fn call(&mut self, _: tonic::Request<DynamicMessage>) -> Self::Future {
        ready(Ok(tonic::Response::new(self.resp_body.clone())))
    }
}

struct InnerServerStreamingService {
    resp_body: DynamicMessage,

    stream_cycle: Option<Duration>,
}

type StreamItem = tonic::Result<DynamicMessage>;

impl ServerStreamingService<DynamicMessage> for InnerServerStreamingService {
    type Response = DynamicMessage;
    type ResponseStream = BoxStream<'static, StreamItem>;
    type Future = BoxFuture<'static, tonic::Result<tonic::Response<Self::ResponseStream>>>;

    fn call(&mut self, _: tonic::Request<DynamicMessage>) -> Self::Future {
        let resp_body = self.resp_body.clone();
        let stream_cycle = self.stream_cycle;
        Box::pin(async move {
            let stream = match stream_cycle {
                Some(cycle) => {
                    let s = IntervalStream::new(time::interval(cycle))
                        .map(move |_| Ok(resp_body.clone()))
                        .boxed();
                    return Ok(tonic::Response::new(s));
                }
                None => stream::once(ready(Ok(resp_body.clone())))
                    .chain(pending())
                    .boxed(),
            };
            Ok(tonic::Response::new(stream))
        })
    }
}

#[derive(Clone, Debug)]
pub struct StaticService {
    codec: DynamicProstCodec,
    served_uri: Uri,
    method_type: MethodDescriptor,
    response: DynamicMessage,

    stream_cycle: Option<Duration>,
}

impl StaticService {
    pub fn new(
        codec: DynamicProstCodec,
        service: &str,
        method: &str,
        method_type: MethodDescriptor,
        response: DynamicMessage,
        stream_cycle: Option<Duration>,
    ) -> anyhow::Result<Self> {
        let served_uri = Uri::from_maybe_shared(format!("/{service}/{method}"))?;

        Ok(Self {
            codec,
            served_uri,
            method_type,
            response,

            stream_cycle,
        })
    }
}

impl<B> Service<Request<B>> for StaticService
where
    B: http_body::Body + Send + 'static,
    B::Error: Into<StdError> + Send + 'static,
{
    type Response = Response<TonicBody>;
    type Error = Infallible;
    type Future = BoxResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        // Check if the request URI matches the served URI
        if req.uri().path_and_query() != self.served_uri.path_and_query() {
            let mut response = Response::new(TonicBody::empty());
            let headers = response.headers_mut();
            headers.insert(
                Status::GRPC_STATUS,
                (tonic::Code::Unimplemented as i32).into(),
            );
            headers.insert(CONTENT_TYPE, GRPC_CONTENT_TYPE);

            return Box::pin(ready(Ok(response)));
        }

        let codec = self.codec.clone();

        if self.method_type.is_server_streaming() {
            let s = InnerServerStreamingService {
                resp_body: self.response.clone(),

                stream_cycle: self.stream_cycle,
            };

            Box::pin(async move { Ok(Grpc::new(codec).server_streaming(s, req).await) })
        } else {
            let s = InnerUnaryService {
                resp_body: self.response.clone(),
            };

            Box::pin(async move { Ok(Grpc::new(codec).unary(s, req).await) })
        }
    }
}
