use crate::{Seconds, SegmentId, TraceId};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, ops::Not};

// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-sendingdata.html
// https://docs.aws.amazon.com/xray/latest/devguide/xray-api-segmentdocuments.html

/// Description of an internal application operation
/// which may be an extension of an external operation
#[derive(Debug, Default, Serialize)]
pub struct Segment {
    /// A unique identifier that connects all segments and subsegments originating from a single client request.
    pub(crate) trace_id: TraceId,
    ///  A 64-bit identifier for the segment, unique among segments in the same trace, in 16 hexadecimal digits.
    pub(crate) id: SegmentId,
    /// The logical name of the service that handled the request, up to 200 characters. For example, your application's name or domain name. Names can contain Unicode letters, numbers, and whitespace, and the following symbols: _, ., :, /, %, &, #, =, +, \, -, @
    ///
    /// A segment's name should match the domain name or logical name of the service that generates the segment. However, this is not enforced. Any application that has permission to PutTraceSegments can send segments with any name.
    pub(crate) name: String,
    /// Number that is the time the segment was created, in floating point seconds in epoch time.
    pub(crate) start_time: Seconds,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Number that is the time the segment was closed.
    pub end_time: Option<Seconds>,
    #[serde(skip_serializing_if = "Not::not")]
    ///  boolean, set to true instead of specifying an end_time to record that a segment is started, but is not complete. Send an in-progress segment when your application receives a request that will take a long time to serve, to trace the request receipt. When the response is sent, send the complete segment to overwrite the in-progress segment. Only send one complete segment, and one or zero in-progress segments, per request.
    pub in_progress: bool,
    /// A subsegment ID you specify if the request originated from an instrumented application. The X-Ray SDK adds the parent subsegment ID to the tracing header for downstream HTTP calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<SegmentId>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws: Option<Aws>,
    /// An object with information about your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

///  An object with information about your application.
#[derive(Debug, Default, Serialize)]
pub struct Service {
    /// A string that identifies the version of your application that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Context information about the AWS environment this segment was run in
#[derive(Debug, Default, Serialize)]
pub struct Aws {
    ///  If your application sends segments to a different AWS account, record the ID of the account running your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    ///  Information about an Amazon ECS container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecs: Option<Ecs>,
    ///  Information about an EC2 instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ec2: Option<Ec2>,
    /// Information about an Elastic Beanstalk environment. You can find this information in a file named /var/elasticbeanstalk/xray/environment.conf on the latest Elastic Beanstalk platforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_beanstalk: Option<ElasticBeanstalk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<Tracing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xray: Option<XRay>,
}

#[derive(Debug, Default, Serialize)]
pub struct XRay {
    pub sdk_version: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Ecs {
    /// The container ID of the container running your application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Ec2 {
    /// The instance ID of the EC2 instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The Availability Zone in which the instance is running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_zone: Option<String>,
}

/// Information about an Elastic Beanstalk environment. You can find this information in a file named /var/elasticbeanstalk/xray/environment.conf on the latest Elastic Beanstalk platforms.
#[derive(Debug, Default, Serialize)]
pub struct ElasticBeanstalk {
    /// The name of the environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_name: Option<String>,
    ///  The name of the application version that is currently deployed to the instance that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_label: Option<String>,
    /// number indicating the ID of the last successful deployment to the instance that served the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<usize>,
}

#[derive(Debug, Default, Serialize)]
pub struct Tracing {
    /// version of sdk
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

/// Detailed representation of an exception
#[derive(Debug, Serialize)]
pub struct Exception {
    /// A 64-bit identifier for the exception, unique among segments in the same trace, in 16 hexadecimal digits.
    pub id: String,
    /// The exception message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<String>,
    /// The exception type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<bool>,
    /// integer indicating the number of stack frames that are omitted from the stack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<usize>,
    ///  integer indicating the number of exceptions that were skipped between this exception and its child, that is, the exception that it caused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<usize>,
    /// Exception ID of the exception's parent, that is, the exception that caused this exception.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// array of stackFrame objects.
    pub stack: Vec<StackFrame>,
}

/// A summary of a single operation within a stack trace
#[derive(Debug, Serialize)]
pub struct StackFrame {
    /// The relative path to the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The line in the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    /// The function or method name.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// Begins a new named segment
    ///
    /// A segment's name should match the domain name or logical name of the service that generates the segment. However, this is not enforced. Any application that has permission to PutTraceSegments can send segments with any name.
    pub fn begin<N>(
        name: N,
        id: SegmentId,
        parent_id: Option<SegmentId>,
        trace_id: TraceId,
    ) -> Self
    where
        N: Into<String>,
    {
        let mut valid_name = name.into();
        if valid_name.len() > 200 {
            valid_name = valid_name[..200].into();
        }
        Segment {
            name: valid_name,
            id,
            parent_id,
            trace_id,
            in_progress: true,
            aws: Some(Aws {
                xray: Some(XRay {
                    sdk_version: Some(env!("CARGO_PKG_VERSION").into()),
                }),
                ..Aws::default()
            }),
            ..Segment::default()
        }
    }

    /// End the segment by assigning its end_time
    pub fn end(&mut self) -> &mut Self {
        self.end_time = Some(Seconds::now());
        self.in_progress = false;
        self
    }
}

/// Describes an http request/response cycle
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Http {
    /// Information about a request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    /// Information about a response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Response>,
}

///  Information about a request.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Request {
    /// The request method. For example, GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// The full URL of the request, compiled from the protocol, hostname, and path of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The IP address of the requester. Can be retrieved from the IP packet's Source Address or, for forwarded requests, from an X-Forwarded-For header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// The user agent string from the requester's client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// (segments only) boolean indicating that the client_ip was read from an X-Forwarded-For header and is not reliable as it could have been forged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_forwarded_for: Option<String>,
    /// (subsegments only) boolean indicating that the downstream call is to another traced service. If this field is set to true, X-Ray considers the trace to be broken until the downstream service uploads a segment with a parent_id that matches the id of the subsegment that contains this block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traced: Option<bool>,
}

///  Information about a response.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
    /// number indicating the HTTP status of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// number indicating the length of the response body in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,
}

impl Subsegment {
    /// Create a new subsegment
    pub fn begin<N>(
        name: N,
        id: SegmentId,
        parent_id: Option<SegmentId>,
        trace_id: TraceId,
    ) -> Self
    where
        N: Into<String>,
    {
        let mut valid_name = name.into();
        if valid_name.len() > 200 {
            valid_name = valid_name[..200].into();
        }
        Subsegment {
            name: valid_name,
            id,
            trace_id: Some(trace_id),
            parent_id,
            type_: "subsegment".into(),
            in_progress: true,
            ..Subsegment::default()
        }
    }

    /// End the subsegment by assigning its end_time
    pub fn end(&mut self) -> &mut Self {
        self.end_time = Some(Seconds::now());
        self.in_progress = false;
        self
    }
}

/// Record information about the AWS services and resources that your application accesses. X-Ray uses this information to create inferred segments that represent the downstream services in your service map.
#[derive(Debug, Default, Serialize)]
pub struct Subsegment {
    /// The logical name of the subsegment. For downstream calls, name the subsegment after the resource or service called. For custom subsegments, name the subsegment after the code that it instruments (e.g., a function name).
    pub(crate) name: String,
    /// A 64-bit identifier for the subsegment, unique among segments in the same trace, in 16 hexadecimal digits.
    pub(crate) id: SegmentId,
    /// number that is the time the subsegment was created, in floating point seconds in epoch time, accurate to milliseconds. For example, 1480615200.010 or 1.480615200010E9.
    pub(crate) start_time: Seconds,
    /// number that is the time the subsegment was closed. For example, 1480615200.090 or 1.480615200090E9. Specify an end_time or in_progress.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) end_time: Option<Seconds>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    ///
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
    /// contents of the sql query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql: Option<Sql>,
}

/// Information about an AWS operation
#[derive(Debug, Default, Serialize)]
pub struct AwsOperation {
    /// The name of the API action invoked against an AWS service or resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// If your application accesses resources in a different account, or sends segments to a different account, record the ID of the account that owns the AWS resource that your application accessed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// If the resource is in a region different from your application, record the region. For example, us-west-2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Unique identifier for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// For operations on an Amazon SQS queue, the queue's URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_url: Option<String>,
    /// For operations on a DynamoDB table, the name of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
}

/// Information about a SQL operation
#[derive(Debug, Default, Serialize)]
pub struct Sql {
    /// For SQL Server or other database connections that don't use URL connection strings, record the connection string, excluding passwords.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,
    /// For a database connection that uses a URL connection string, record the URL, excluding passwords.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The database query, with any user provided values removed or replaced by a placeholder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitized_query: Option<String>,
    /// The name of the database engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_type: Option<String>,
    /// The version number of the database engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_version: Option<String>,
    /// The name and version number of the database engine driver that your application uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    /// The database username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// call if the query used a PreparedCall; statement if the query used a PreparedStatement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Seconds, Segment, SegmentId, Subsegment, TraceId};

    #[test]
    fn segments_begin_with_names_with_a_max_len() {
        assert_eq!(
            Segment::begin("short", SegmentId::default(), None, TraceId::default()).name,
            "short"
        );
        assert_eq!(
            Segment::begin(
                String::from_utf8_lossy(&[b'X'; 201]),
                SegmentId::default(),
                None,
                TraceId::default()
            )
            .name
            .len(),
            200
        );
    }

    #[test]
    fn subsegments_begin_with_names_with_a_max_len() {
        assert_eq!(
            Subsegment::begin("short", SegmentId::default(), None, TraceId::default()).name,
            "short"
        );
        assert_eq!(
            Subsegment::begin(
                String::from_utf8_lossy(&[b'X'; 201]),
                SegmentId::default(),
                None,
                TraceId::default()
            )
            .name
            .len(),
            200
        );
    }

    #[test]
    fn segments_serialize() {
        assert_eq!(
            r#"{"trace_id":"1-581cf771-a006649127e371903a2de979","id":"70de5b6f19ff9a0a","name":"Scorekeep","start_time":1478293361.271,"end_time":1478293361.449}"#,
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
