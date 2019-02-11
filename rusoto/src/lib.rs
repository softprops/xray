//! AWS X-Ray tracing integration for for the Rusoto AWS SDK

use futures::Future;
use rusoto_core::{
    request::{HttpClient, HttpDispatchError, HttpResponse},
    signature::SignedRequest,
    DispatchSignedRequest,
};
use std::{sync::Arc, time::Duration};
use xray::{Client, Segment, Subsegment};

pub struct TracedRequests<D> {
    dispatcher: D,
    client: Arc<Client>,
}

impl<D> TracedRequests<D> {
    /// Create a new tracing dispatcher with a default X-Ray client
    pub fn new(dispatcher: D) -> Self {
        Self::new_with_client(dispatcher, Arc::new(Client::default()))
    }

    /// Create a new tracing dispatcher with a custom X-Ray client
    pub fn new_with_client(
        dispatcher: D,
        client: Arc<Client>,
    ) -> Self {
        Self { dispatcher, client }
    }
}

impl Default for TracedRequests<HttpClient> {
    fn default() -> Self {
        TracedRequests::new(HttpClient::new().expect("failed to initialize client"))
    }
}

/// Implementation of DispatchSignedRequest which wraps
/// an implementation of another DispatchSignedRequest
/// with a tracing future
impl<D> DispatchSignedRequest for TracedRequests<D>
where
    D: DispatchSignedRequest + Send + Sync + 'static,
    D::Future: Send,
{
    type Future = TracingRequest<D::Future>;
    fn dispatch(
        &self,
        request: SignedRequest,
        timeout: Option<Duration>,
    ) -> Self::Future {
        let segment = Segment::begin("test");
        let mut subsegment =
            Subsegment::begin(segment.trace_id.clone(), None, request.service.as_str());
        subsegment.namespace = Some("aws".into());
        TracingRequest(
            self.dispatcher.dispatch(request, timeout),
            subsegment,
            self.client.clone(),
        )
    }
}

/** a dispatching request that will be traced if x-ray trace is sampled */
pub struct TracingRequest<T>(T, Subsegment, Arc<Client>);

impl<T> Future for TracingRequest<T>
where
    T: Future<Item = HttpResponse, Error = HttpDispatchError> + Send,
{
    type Item = HttpResponse;
    type Error = HttpDispatchError;
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(futures::Async::Ready(res)) => {
                // todo: add tracing
                Ok(futures::Async::Ready(res))
            }
            err @ Err(_) => err,
            other => other,
        }
    }
}
