use std::{
    convert::Infallible,
    future::{ready, Ready},
    task::{Context, Poll}, thread::sleep, time::Duration,
};

use futures_util::future::BoxFuture;
use http::{Request, Response, Uri, header::CONTENT_TYPE};
use prost_reflect::DynamicMessage;
use tonic::{
    Status,
    body::Body as TonicBody,
    metadata::GRPC_CONTENT_TYPE,
    server::{Grpc, UnaryService},
};
use tower_service::Service;

use crate::codec::DynamicProstCodec;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type BoxResultFuture<T, E> = BoxFuture<'static, Result<T, E>>;

#[derive(Clone, Debug)]
pub struct StaticService {
    codec: DynamicProstCodec,
    served_uri: Uri,
    response: DynamicMessage,
}

impl StaticService {
    pub fn new(
        codec: DynamicProstCodec,
        service: &str,
        method: &str,
        response: DynamicMessage,
    ) -> anyhow::Result<Self> {
        let served_uri = Uri::from_maybe_shared(format!("/{service}/{method}"))?;

        Ok(Self {
            codec,
            served_uri,
            response,
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

        struct InnerService {
            resp_body: DynamicMessage,
        }

        impl UnaryService<DynamicMessage> for InnerService {
            type Response = DynamicMessage;
            type Future = Ready<Result<tonic::Response<Self::Response>, Status>>;

            fn call(&mut self, _: tonic::Request<DynamicMessage>) -> Self::Future {
                ready(Ok(tonic::Response::new(self.resp_body.clone())))
            }
        }

        let codec = self.codec.clone();
        let inner_service = InnerService {
            resp_body: self.response.clone(),
        };

        Box::pin(async move { Ok(Grpc::new(codec).unary(inner_service, req).await) })
    }
}
