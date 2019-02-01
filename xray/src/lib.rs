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
    ///
    /// A segment's name should match the domain name or logical name of the service that generates the segment. However, this is not enforced. Any application that has permission to PutTraceSegments can send segments with any name.
    pub name: String,
    /// Number that is the time the segment was created, in floating point seconds in epoch time.
    pub start_time: Seconds,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Number that is the time the segment was closed.
    pub end_time: Option<Seconds>,
    #[serde(skip_serializing_if = "Not::not")]
    ///  boolean, set to true instead of specifying an end_time to record that a segment is started, but is not complete. Send an in-progress segment when your application receives a request that will take a long time to serve, to trace the request receipt. When the response is sent, send the complete segment to overwrite the in-progress segment. Only send one complete segment, and one or zero in-progress segments, per request.
    pub in_progress: bool,
    /// A subsegment ID you specify if the request originated from an instrumented application. The X-Ray SDK adds the parent subsegment ID to the tracing header for downstream HTTP calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Indicates that a server error occurred (response status code was 5XX Server Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub fault: bool,
    /// Indicates that a client error occurred (response status code was 4XX Client Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub error: bool,
    /// boolean indicating that a request was throttled (response status code was 429 Too Many Requests).
    #[serde(skip_serializing_if = "Not::not")]
    pub throttle: bool,
    ///  error fields that indicate an error occurred and that include information about the exception that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    /// The type of AWS resource running your application.
    /// todo: convert to enum (see aws docs for values)
    /// When multiple values are applicable to your application, use the one that is most specific. For example, a Multicontainer Docker Elastic Beanstalk environment runs your application on an Amazon ECS container, which in turn runs on an Amazon EC2 instance. In this case you would set the origin to AWS::ElasticBeanstalk::Environment as the environment is the parent of the other two resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// A string that identifies the user who sent the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_arn: Option<String>,
    /// http objects with information about the original HTTP request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Http>,
    /// annotations object with key-value pairs that you want X-Ray to index for search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, Annotation>>,
    /// metadata object with any additional data that you want to store in the segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// aws object with information about the AWS resource on which your application served the request.
    pub aws: Option<Aws>,
    /// An object with information about your application.
    pub service: Option<Service>,
}

///  An object with information about your application.
#[derive(Debug, Default, Serialize)]
pub struct Service {
    /// A string that identifies the version of your application that served the request.
    pub version: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Aws {
    ///  If your application sends segments to a different AWS account, record the ID of the account running your application.
    pub account_id: Option<String>,
    ///  Information about an Amazon ECS container.
    pub ecs: Option<Ecs>,
    ///  Information about an EC2 instance.
    pub ec2: Option<Ec2>,
    /// Information about an Elastic Beanstalk environment. You can find this information in a file named /var/elasticbeanstalk/xray/environment.conf on the latest Elastic Beanstalk platforms.
    pub elastic_beanstalk: Option<ElasticBeanstalk>,
    pub tracing: Option<Tracing>,
}

#[derive(Debug, Default, Serialize)]
pub struct Ecs {
    /// The container ID of the container running your application.
    pub container: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Ec2 {
    /// The instance ID of the EC2 instance.
    pub instance_id: Option<String>,
    /// The Availability Zone in which the instance is running.
    pub availability_zone: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct ElasticBeanstalk {
    /// The name of the environment.
    pub environment_name: Option<String>,
    ///  The name of the application version that is currently deployed to the instance that served the request.
    pub version_label: Option<String>,
    /// number indicating the ID of the last successful deployment to the instance that served the request.
    pub deployment_id: Option<usize>,
}

#[derive(Debug, Default, Serialize)]
pub struct Tracing {
    pub sdk: Option<String>,
}

impl Default for Annotation {
    fn default() -> Self {
        Annotation::String("".into())
    }
}

/// A value type which may be used for
/// filter querying
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Annotation {
    /// A string value
    String(String),
    /// A numberic value
    Number(usize),
    /// A boolean value
    Bool(bool),
}

#[derive(Debug, Serialize)]
pub struct Exception {
    /// A 64-bit identifier for the exception, unique among segments in the same trace, in 16 hexadecimal digits.
    pub id: String,
    /// The exception message.
    pub messages: Option<String>,
    /// The exception type.
    pub remote: Option<bool>,
    /// integer indicating the number of stack frames that are omitted from the stack.
    pub truncated: Option<usize>,
    ///  integer indicating the number of exceptions that were skipped between this exception and its child, that is, the exception that it caused.
    pub skipped: Option<usize>,
    /// Exception ID of the exception's parent, that is, the exception that caused this exception.
    pub cause: Option<String>,
    /// array of stackFrame objects.
    pub stack: Vec<StackFrame>,
}

#[derive(Debug, Serialize)]
pub struct StackFrame {
    /// The relative path to the file.
    pub path: Option<String>,
    /// The line in the file.
    pub line: Option<String>,
    /// The function or method name.
    pub label: Option<String>,
}

/// Represents the cause of an errror
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Cause {
    ///  a 16 character exception ID
    Name(String),
    /// A description of an error
    Description {
        ///  The full path of the working directory when the exception occurred.
        working_directory: String,
        ///  The array of paths to libraries or modules in use when the exception occurred.
        paths: Vec<String>,
        /// The array of exception objects.
        exceptions: Vec<Exception>,
    },
}

impl Segment {
    /// Begins a new segment
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

/// Describes an http request/response cycle
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Http {
    /// Information about a request
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<Request>,
    /// Information about a response.
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
    /// The IP address of the requester. Can be retrieved from the IP packet's Source Address or, for forwarded requests, from an X-Forwarded-For header.
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ip: Option<String>,
    /// The user agent string from the requester's client.
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
    /// (segments only) boolean indicating that the client_ip was read from an X-Forwarded-For header and is not reliable as it could have been forged.
    #[serde(skip_serializing_if = "Option::is_none")]
    x_forwarded_for: Option<String>,
    /// (subsegments only) boolean indicating that the downstream call is to another traced service. If this field is set to true, X-Ray considers the trace to be broken until the downstream service uploads a segment with a parent_id that matches the id of the subsegment that contains this block.
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

#[derive(Debug, Default, Serialize)]
pub struct Subsegment {
    /// The logical name of the subsegment. For downstream calls, name the subsegment after the resource or service called. For custom subsegments, name the subsegment after the code that it instruments (e.g., a function name).
    pub name: String,
    /// A 64-bit identifier for the subsegment, unique among segments in the same trace, in 16 hexadecimal digits.
    pub id: SegmentId,
    /// number that is the time the subsegment was created, in floating point seconds in epoch time, accurate to milliseconds. For example, 1480615200.010 or 1.480615200010E9.
    pub start_time: Seconds,
    ///  number that is the time the subsegment was closed. For example, 1480615200.090 or 1.480615200090E9. Specify an end_time or in_progress.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Seconds>,
    /// Trace ID of the subsegment's parent segment. Required only if sending a subsegment separately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
    /// Segment ID of the subsegment's parent segment. Required only if sending a subsegment separately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<SegmentId>,
    /// boolean that is set to true instead of specifying an end_time to record that a subsegment is started, but is not complete. Only send one complete subsegment, and one or zero in-progress subsegments, per downstream request.
    #[serde(skip_serializing_if = "Not::not")]
    pub in_progress: bool,
    /// boolean indicating that a server error occurred (response status code was 5XX Server Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub fault: bool,
    /// boolean indicating that a client error occurred (response status code was 4XX Client Error).
    #[serde(skip_serializing_if = "Not::not")]
    pub error: bool,
    ///  boolean indicating that a request was throttled (response status code was 429 Too Many Requests).
    #[serde(skip_serializing_if = "Not::not")]
    pub throttled: bool,
    /// aws for AWS SDK calls; remote for other downstream calls.
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traced: Option<bool>,
    /// array of subsegment IDs that identifies subsegments with the same parent that completed prior to this subsegment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precursor_ids: Option<Vec<String>>,
    /// information about the cause of an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    /// annotations object with key-value pairs that you want X-Ray to index for search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, Annotation>>,
    /// metadata object with any additional data that you want to store in the segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// subsegment. Required only if sending a subsegment separately.
    #[serde(rename = "type")]
    pub type_: String,
    /// array of subsegment objects.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subsegments: Vec<Subsegment>,
    ///  http object with information about an outgoing HTTP call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Http>,
    /// aws object with information about the downstream AWS resource that your application called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws: Option<AwsOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql: Option<Sql>,
}

/// Information about an AWS operation
#[derive(Debug, Default, Serialize)]
pub struct AwsOperation {
    /// The name of the API action invoked against an AWS service or resource.
    pub operation: Option<String>,
    /// If your application accesses resources in a different account, or sends segments to a different account, record the ID of the account that owns the AWS resource that your application accessed.
    pub account_id: Option<String>,
    /// If the resource is in a region different from your application, record the region. For example, us-west-2.
    pub region: Option<String>,
    /// Unique identifier for the request.
    pub request_id: Option<String>,
    /// For operations on an Amazon SQS queue, the queue's URL.
    pub queue_url: Option<String>,
    /// For operations on a DynamoDB table, the name of the table.
    pub table_name: Option<String>,
}

/// Information about a SQL operation
#[derive(Debug, Default, Serialize)]
pub struct Sql {
    /// For SQL Server or other database connections that don't use URL connection strings, record the connection string, excluding passwords.
    pub connection_string: Option<String>,
    /// For a database connection that uses a URL connection string, record the URL, excluding passwords.
    pub url: Option<String>,
    /// The database query, with any user provided values removed or replaced by a placeholder.
    pub sanitized_query: Option<String>,
    /// The name of the database engine.
    pub database_type: Option<String>,
    /// The version number of the database engine.
    pub database_version: Option<String>,
    /// The name and version number of the database engine driver that your application uses.
    pub driver_version: Option<String>,
    /// The database username.
    pub user: Option<String>,
    /// call if the query used a PreparedCall; statement if the query used a PreparedStatement.
    pub preparation: Option<String>,
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
