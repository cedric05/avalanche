use std::{future::Future, pin::Pin};

use http::{Request, Response};
use tower::{Service, Layer};

#[derive(Clone)]
pub(crate) struct NoAuth<S> {
    inner: S,
}

struct NoAuthLayer;

impl<S> Layer<S> for NoAuthLayer {
    type Service = NoAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NoAuth { inner }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for NoAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: 'static,
    <S as Service<Request<ReqBody>>>::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let fut = self.inner.call(req);
        Box::pin(async { fut.await })
    }
}
