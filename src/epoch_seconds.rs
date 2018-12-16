use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{de, ser, Serializer};
use std::fmt;

/// Represents fractional seconds since the epoch
/// These can be derived from std::time::Duration and be converted
/// too std::time::Duration
///
/// A Default implementation is provided which yields the number of seconds since the epoch from
/// the system time's `now` value
#[derive(Debug)]
pub struct EpochSeconds(f64);

impl EpochSeconds {
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .into()
    }
}

impl Default for EpochSeconds {
  fn default() -> Self {
    EpochSeconds::now()
  }
}

impl From<Duration> for EpochSeconds {
    fn from(d: Duration) -> Self {
        EpochSeconds(d.as_secs() as f64 + (f64::from(d.subsec_nanos()) / 1.0e9))
    }
}

impl Into<Duration> for EpochSeconds {
    fn into(self) -> Duration {
        let EpochSeconds(secs) = self;
        Duration::new(secs.trunc() as u64, (secs.fract() * 1.0e9) as u32)
    }
}

struct EpochSecondsVisitor;

impl<'de> de::Visitor<'de> for EpochSecondsVisitor {
    type Value = EpochSeconds;

    fn expecting(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        formatter.write_str("a string value")
    }
    fn visit_f64<E>(
        self,
        value: f64,
    ) -> Result<EpochSeconds, E>
    where
        E: de::Error,
    {
        Ok(EpochSeconds(value))
    }
}

impl ser::Serialize for EpochSeconds {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
      let EpochSeconds(seconds) = self;
        serializer.serialize_f64(*seconds)
    }
}

impl<'de> de::Deserialize<'de> for EpochSeconds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_f64(EpochSecondsVisitor)
    }
}