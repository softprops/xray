#![warn(missing_docs)]
//#![deny(warnings)]
//! Provides a client interface for [AWS X-Ray](https://aws.amazon.com/xray/)
// Std
use std::{
    cell::RefCell, collections::HashMap, env, net::SocketAddr, ops::Not,
    result::Result as StdResult,
};

// Third Party;
use tokio::net::UdpSocket;

mod epoch;
mod error;
mod hexbytes;
mod segment_id;
mod trace_id;

pub use crate::{epoch::Seconds, error::Error, segment_id::SegmentId, trace_id::TraceId};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

const HEADER: &[u8] = br#"{"format": "json", "version": 1}\n"#;

/// Type alias for Results which may return `xray::Errors`
pub type Result<T> = StdResult<T, Error>;

/// X-Ray daemon client interface
#[derive(Debug)]
pub struct Client {
    addr: SocketAddr,
    socket: RefCell<UdpSocket>,
}

impl Default for Client {
    fn default() -> Self {
        // https://docs.aws.amazon.com/lambda/latest/dg/lambda-x-ray.html
        // todo documment error handling
        let addr: SocketAddr = env::var("AWS_XRAY_DAEMON_ADDRESS")
            .map_err(|_| ())
            .and_then(|value| value.parse::<SocketAddr>().map_err(|_| ()))
            .unwrap_or_else(|_| {
                log::trace!("No valid `AWS_XRAY_DAEMON_ADDRESS` env variable detected falling back on default");
                ([127, 0, 0, 1], 2000).into()
            });

        Client::new(addr).unwrap()
    }
}

impl Client {
    /// Return a new X-Ray client connected
    /// to the provided `addr`
    pub fn new(addr: SocketAddr) -> Result<Self> {
        let socket = RefCell::new(
            UdpSocket::bind(&([0, 0, 0, 0], 0).into()).expect("Failed to bind to udp socket"),
        );

        socket.borrow_mut().connect(&addr)?;
        Ok(Client { addr, socket })
    }

    /// send a segment to the xray daemon this client is connected to
    pub fn send(
        &self,
        value: &Segment,
    ) -> Result<()> {
        // todo rep error
        // https://github.com/tokio-rs/tokio/blob/master/examples/udp-client.rs#L44
        let bytes = serde_json::to_vec(&value)?;
        let packet = [HEADER, &bytes].concat();
        self.socket.borrow_mut().poll_send(&packet)?;
        Ok(())
    }
}

/*pub fn epoch_seconds() -> f64 {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    d.as_secs() as f64 + (f64::from(d.subsec_nanos()) / 1.0e9)
}*/

// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-sendingdata.html
// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-segmentdocuments.html

/// Description of an internal application operation
/// which may be an extension of an external operation
#[derive(Debug, Default, Serialize)]
pub struct Segment {
    /// A unique identifier that connects all segments and subsegments originating from a single client request.
    pub trace_id: TraceId,
    ///  A 64-bit identifier for the segment, unique among segments in the same trace, in 16 hexadecimal digits.
    pub id: SegmentId,
    /// The logical name of the service that handled the request, up to 200 characters. For example, your application's name or domain name. Names can contain Unicode letters, numbers, and whitespace, and the following symbols: _, ., :, /, %, &, #, =, +, \, -, @
    pub name: String,
    ///  number that is the time the segment was created, in floating point seconds in epoch time.
    pub start_time: Seconds,
    #[serde(skip_serializing_if = "Option::is_none")]
    ///  number that is the time the segment was closed.
    pub end_time: Option<Seconds>,
    #[serde(skip_serializing_if = "Not::not")]
    ///  boolean, set to true instead of specifying an end_time to record that a segment is started, but is not complete. Send an in-progress segment when your application receives a request that will take a long time to serve, to trace the request receipt. When the response is sent, send the complete segment to overwrite the in-progress segment. Only send one complete segment, and one or zero in-progress segments, per request.
    pub in_progress: bool,
    /// A subsegment ID you specify if the request originated from an instrumented application. The X-Ray SDK adds the parent subsegment ID to the tracing header for downstream HTTP calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// boolean indicating that a server error occurred (response status code was 5XX Server Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub fault: bool,
    /// boolean indicating that a client error occurred (response status code was 4XX Client Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub error: bool,
    /// boolean indicating that a request was throttled (response status code was 429 Too Many Requests).
    #[serde(skip_serializing_if = "Not::not")]
    pub throttle: bool,
    ///  error fields that indicate an error occurred and that include information about the exception that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    /// The type of AWS resource running your application.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, Annotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>, //pub aws:  ???,
                                                  //pub service: Option<Service>,
                                                  //pub SQLData: Option<Sql>,
                                                  //pub annotations: Option<BTreeMap<String, Value>>,
                                                  //pub metadata: Option<BTreeMap<String, Value>>,
                                                  //pub subsegments: Option<Value>
}

impl Default for Annotation {
    fn default() -> Self {
        Annotation::String("".into())
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Annotation {
    String(String),
    Number(usize),
    Bool(bool),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Cause {
    Name(String),
    Description {
        working_directory: String,
        paths: Vec<String>,
        // exceptions: Vec<???>
    },
}

impl Segment {
    pub fn begin<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Segment {
            name: name.into(),
            ..Segment::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Http {
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<Request>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<Response>,
}

///  Information about a request.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Request {
    /// The request method. For example, GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    /// The full URL of the request, compiled from the protocol, hostname, and path of the request.
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

///  Information about a response.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
    /// number indicating the HTTP status of the response.
    status: Option<u16>,
    /// number indicating the length of the response body in bytes.
    content_length: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::{Seconds, Segment, SegmentId, TraceId};

    #[test]
    fn segments_serialize() {
        println!(
            "{}",
            serde_json::to_string(&Segment {
                name: "Scorekeep".into(),
                id: SegmentId::Rendered("70de5b6f19ff9a0a".into()),
                start_time: Seconds(1_478_293_361.271),
                trace_id: TraceId::Rendered("1-581cf771-a006649127e371903a2de979".into()),
                end_time: Some(Seconds(1_478_293_361.449)),
                ..Segment::default()
            })
            .expect("failed to serialize")
        )
    }
}
