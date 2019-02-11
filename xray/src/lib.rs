#![warn(missing_docs)]
//#![deny(warnings)]
//! Provides a client interface for [AWS X-Ray](https://aws.amazon.com/xray/)

use serde::Serialize;
use std::{
    env,
    net::{SocketAddr, UdpSocket},
    result::Result as StdResult,
    sync::Arc,
};

mod epoch;
mod error;
mod header;
mod hexbytes;
mod lambda;
mod segment;
mod segment_id;
mod trace_id;

pub use crate::{
    epoch::Seconds, error::Error, header::Header, segment::*, segment_id::SegmentId,
    trace_id::TraceId,
};

/// Type alias for Results which may return `xray::Errors`
pub type Result<T> = StdResult<T, Error>;

/// X-Ray daemon client interface
#[derive(Debug)]
pub struct Client {
    addr: SocketAddr,
    socket: Arc<UdpSocket>,
}

impl Default for Client {
    /// Return a client configured to send trace data to an
    /// address identified by a `AWS_XRAY_DAEMON_ADDRESS` env variable
    /// or `127.0.0.1:2000`
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
    const HEADER: &'static [u8] = br#"{"format": "json", "version": 1}\n"#;

    /// Return a new X-Ray client connected
    /// to the provided `addr`
    pub fn new(addr: SocketAddr) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(&[([0, 0, 0, 0], 0).into()][..])?);
        socket.set_nonblocking(true)?;
        socket.connect(&addr)?;
        Ok(Client { addr, socket })
    }

    #[inline]
    fn packet<S>(data: S) -> Result<Vec<u8>>
    where
        S: Serialize,
    {
        let bytes = serde_json::to_vec(&data)?;
        Ok([Self::HEADER, &bytes].concat())
    }

    /// send a segment to the xray daemon this client is connected to
    pub fn send<S>(
        &self,
        data: &S,
    ) -> Result<()>
    where
        S: Serialize,
    {
        self.socket.send(&Self::packet(data)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_prefixes_packets_with_header() {
        assert_eq!(
            Client::packet(serde_json::json!({
                "foo": "bar"
            }))
            .unwrap(),
            br#"{"format": "json", "version": 1}\n{"foo":"bar"}"#.to_vec()
        )
    }
}
