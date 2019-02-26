//! AWS X-Ray tracing integration for for the Rusoto AWS SDK

use futures::Future;
use rusoto_core::{
    request::{HttpClient, HttpDispatchError, HttpResponse},
    signature::SignedRequest,
    DispatchSignedRequest,
};
use std::time::Duration;
use xray::{
    segment::{AwsOperation, Http, Response},
    OpenSubsegment, Recorder,
};

pub struct TracedRequests<D> {
    dispatcher: D,
    recorder: Recorder,
}

impl<D> TracedRequests<D> {
    /// Create a new tracing dispatcher with a default X-Ray client
    pub fn new(dispatcher: D) -> Self {
        Self::new_with_recorder(dispatcher, Recorder::default())
    }

    /// Create a new tracing dispatcher with a custom X-Ray client
    pub fn new_with_recorder(
        dispatcher: D,
        recorder: Recorder,
    ) -> Self {
        Self {
            dispatcher,
            recorder,
        }
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
        let mut open = self.recorder.begin_subsegment(request.service.as_ref());
        let operation = request
            .headers
            .get("x-amz-target")
            .and_then(|values| values.iter().next())
            .and_then(|value| {
                value
                    .iter()
                    .position(|&r| r == b'.')
                    .and_then(|pos| String::from_utf8(value[pos..].to_vec()).ok())
            });
        let region = Some(request.region.name().into());
        if let Some(sub) = open.subsegment() {
            sub.aws = Some(AwsOperation {
                operation,
                region,
                ..AwsOperation::default()
            });
        };

        if let Some(seg) = open.subsegment() {
            // populate subsegment fields
            seg.namespace = Some("aws".into());
        }
        TracingRequest(
            self.dispatcher.dispatch(request, timeout),
            self.recorder.clone(),
            open,
        )
    }
}

/** a dispatching request that will be traced if x-ray trace is sampled */
pub struct TracingRequest<T>(T, Recorder, OpenSubsegment);

impl<T> Future for TracingRequest<T>
where
    T: Future<Item = HttpResponse, Error = HttpDispatchError> + Send,
{
    type Item = HttpResponse;
    type Error = HttpDispatchError;
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(futures::Async::Ready(res)) => {
                if let Some(sub) = self.2.subsegment() {
                    sub.http = Some(Http {
                        response: Some(Response {
                            status: Some(res.status.as_u16()),
                            content_length: res
                                .headers
                                .get("Content-Length")
                                .and_then(|value| value.parse::<u64>().ok()),
                        }),
                        ..Http::default()
                    });
                }
                Ok(futures::Async::Ready(res))
            }
            err @ Err(_) => err,
            other => other,
        }
    }
}
