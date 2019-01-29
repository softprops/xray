use crate::hexbytes::Bytes;
use rand::RngCore;
use serde::{de, ser, Serializer};
use std::fmt;

/// Unique identifier of an operation within a trace
#[derive(Debug, PartialEq)]
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
            SegmentId::New(bytes) => write!(f, "{:x}", Bytes(bytes)),
            SegmentId::Rendered(value) => write!(f, "{}", value),
        }
    }
}

impl Default for SegmentId {
    fn default() -> Self {
        SegmentId::new()
    }
}

struct SegmentIdVisitor;

impl<'de> de::Visitor<'de> for SegmentIdVisitor {
    type Value = SegmentId;

    fn expecting(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        formatter.write_str("a string value")
    }
    fn visit_str<E>(
        self,
        value: &str,
    ) -> Result<SegmentId, E>
    where
        E: de::Error,
    {
        Ok(SegmentId::Rendered(value.into()))
    }
}

impl ser::Serialize for SegmentId {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> de::Deserialize<'de> for SegmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(SegmentIdVisitor)
    }
}
