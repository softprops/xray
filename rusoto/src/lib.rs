//! AWS X-Ray tracing integration for for the Rusoto AWS SDK

use futures::Future;
use rusoto_core::{
    request::{HttpClientFuture, HttpDispatchError, HttpResponse},
    signature::SignedRequest,
    DispatchSignedRequest, RusotoFuture,
};
use std::{sync::Arc, time::Duration};
use xray::{Header, Client, Segment, Subsegment};

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
    pub fn new_with_client(dispatcher: D, client: Arc<Client>) -> Self {
        Self { dispatcher, client }
    }
}

pub struct TracingRequest<T>(T, Subsegment, Arc<Client>);

impl<T> Future for TracingRequest<T>
where
    T: Future<Item = HttpResponse, Error = HttpDispatchError> + Send,
{
    type Item = HttpResponse;
    type Error = HttpDispatchError;
    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Err(e) => Err(e),
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Ok(futures::Async::Ready(res)) => {
                let mut ss = &mut self.1;
                ss.end();
                //self.2.send(ss);
                Ok(futures::Async::Ready(res))
            }
        }
    }
}

impl<D> DispatchSignedRequest for TracedRequests<D>
where
    D: DispatchSignedRequest + Send + Sync + 'static,
    D::Future: Send,
{
    type Future = TracingRequest<D::Future>;
    fn dispatch(&self, request: SignedRequest, timeout: Option<Duration>) -> Self::Future {
        println!("{:#?}", request);
        // https://github.com/aws/aws-xray-sdk-go/blob/master/xray/aws.go#L58
        let segment = Segment::begin("test-service");
        let mut subsegment =
            Subsegment::begin(segment.trace_id.clone(), None, request.service.as_str());
        subsegment.namespace = Some("aws".into());
        //request.add_header(Header::NAME, &format!("{}", Header::new(segment.trace_id)));
        println!("{:#?}", request);
        TracingRequest(
            self.dispatcher.dispatch(request, timeout),
            subsegment,
            self.client.clone(),
        )
    }
}
