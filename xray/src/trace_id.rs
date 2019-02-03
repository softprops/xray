use crate::{epoch::Seconds, hexbytes::Bytes};
use rand::RngCore;
use serde::{de, ser, Serializer};
use std::fmt;
/// Coorelates a string of spans together
///
/// Users need only refer to displability
/// a factory for generating these is provided.
///
///
#[derive(Debug, PartialEq, Clone)]
pub enum TraceId {
    #[doc(hidden)]
    New(u64, [u8; 12]),
    #[doc(hidden)]
    Rendered(String),
}

impl TraceId {
    /// Generate a new random trace ID
    pub fn new() -> Self {
        let mut buf = [0; 12];
        rand::thread_rng().fill_bytes(&mut buf);
        TraceId::New(Seconds::now().trunc(), buf)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        TraceId::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceId::New(seconds, bytes) => write!(f, "1-{:08x}-{:x}", seconds, Bytes(bytes)),
            TraceId::Rendered(value) => write!(f, "{}", value),
        }
    }
}

struct TraceIdVisitor;

impl<'de> de::Visitor<'de> for TraceIdVisitor {
    type Value = TraceId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string value")
    }
    fn visit_str<E>(self, value: &str) -> Result<TraceId, E>
    where
        E: de::Error,
    {
        Ok(TraceId::Rendered(value.into()))
    }
}

impl ser::Serialize for TraceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> de::Deserialize<'de> for TraceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(TraceIdVisitor)
    }
}
