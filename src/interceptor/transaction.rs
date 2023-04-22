use hyper::Body;
use tonic::body::BoxBody;
use tower::{Layer, Service};

#[derive(Debug, Clone, Default)]
pub struct TransactionMiddlewareLayer;

impl<S> Layer<S> for TransactionMiddlewareLayer {
    type Service = TransactionMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        TransactionMiddleware { inner: service }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionMiddleware<S> {
    inner: S,
}

impl<S> Service<hyper::Request<Body>> for TransactionMiddleware<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: hyper::Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let response = inner.call(req).await?;

            Ok(response)
        })
    }
}
