//! Provides a client interface for [AWS X-Ray](https://aws.amazon.com/xray/)
// Std
use std::{
    fmt,
    ops::Not,
    time::{SystemTime, UNIX_EPOCH},
};

// Third Party
use rand::RngCore;
use serde_derive::{Deserialize, Serialize};

mod bytebuf;
use crate::bytebuf::ByteBuf;

/// Coorelates a string of spans together
///
/// Users need only refer to displayability
/// a factory for generating these is provided.
///
///
#[derive(Debug)]
pub enum TraceId {
    New(u64, [u8; 12]),
    Rendered(String),
}

impl TraceId {
    pub fn new() -> Self {
        let mut buf = [0; 12];
        rand::thread_rng().fill_bytes(&mut buf);
        TraceId::New(unix_seconds(), buf)
    }
}

impl fmt::Display for TraceId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            TraceId::New(seconds, bytes) => write!(f, "1-{:08x}-{:x}", seconds, ByteBuf(bytes)),
            TraceId::Rendered(value) => write!(f, "{}", value),
        }
    }
}

/// Unique identifier of an operation within a trace
#[derive(Debug)]
pub enum SegmentId {
    New([u8; 8]),
    Rendered(String),
}

impl SegmentId {
    pub fn new() -> Self {
        let mut buf = [0; 8];
        rand::thread_rng().fill_bytes(&mut buf);
        SegmentId::New(buf)
    }
}

impl fmt::Display for SegmentId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            SegmentId::New(bytes) => write!(f, "{:x}", ByteBuf(bytes)),
            SegmentId::Rendered(value) => write!(f, "{}", value),
        }
        //write!(f, "{:x}", ByteBuf(self.0.as_bytes()))
        //write!(f, "{}", self.0)
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}


// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-sendingdata.html
// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-segmentdocuments.html

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub trace_id: Option<String>,
    pub id: String,
    pub name: String,
    pub start_time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    #[serde(skip_serializing_if = "Not::not")]
    pub in_progress: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub fault: bool,
    pub error: bool,
    pub throttle: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precursor_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Http>,
    //pub aws:  ???,
    //pub service: Option<Service>,
    //pub SQLData: Option<Sql>,
    //pub annotations: Option<BTreeMap<String, Value>>,
    //pub metadata: Option<BTreeMap<String, Value>>,
    //pub subsegments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cause {
    #[serde(skip_serializing_if = "Option::is_none")]
    working_directory: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paths: Vec<String>,
    //   exceptions: Vec<Exception>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Http {
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<Request>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<Response>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x_forwarded_for: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    traced: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    status: Option<u16>,
    content_length: Option<u64>,
}

pub fn trace_id() -> String {
    //let mut buf = [0; 12];
    //rand::thread_rng().fill_bytes(&mut buf);
    //TraceId(unix_seconds(), &buf)
    //format!("1-{:08x}-{:x}", unix_seconds(), ByteBuf(&buf))
    format!("{}", TraceId::new())
}

pub fn segment_id() -> String {
    let mut buf = [0; 8];
    rand::thread_rng().fill_bytes(&mut buf);
    //SpanId(&buf)
    format!("{:x}", ByteBuf(&buf))
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
