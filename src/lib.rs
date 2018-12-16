//! Provides a client interface for [AWS X-Ray](https://aws.amazon.com/xray/)
// Std
use std::{
    cell::RefCell,
    env,
    net::SocketAddr,
    ops::Not
};

// Third Party;
use tokio::net::UdpSocket;

mod epoch_seconds;
mod error;
mod hexbytes;
mod segment_id;
mod trace_id;

use crate::epoch_seconds::EpochSeconds;
use crate::{error::Error, segment_id::SegmentId, trace_id::TraceId};
use serde_derive::{Deserialize, Serialize};

const HEADER: &[u8] = br#"{"format": "json", "version": 1}\n"#;

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Segment {
    pub trace_id: TraceId,
    pub id: SegmentId,
    pub name: String,
    pub start_time: EpochSeconds,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<EpochSeconds>,
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
pub struct Cause {
    #[serde(skip_serializing_if = "Option::is_none")]
    working_directory: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paths: Vec<String>,
    //   exceptions: Vec<Exception>
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Http {
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<Request>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<Response>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
    status: Option<u16>,
    content_length: Option<u64>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
