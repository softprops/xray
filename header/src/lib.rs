//! X-Ray [tracing header](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html?shortFooter=true#xray-concepts-tracingheader)
//! parser

use std::{collections::HashMap, str::FromStr};

pub const NAME: &str = "X-Amzn-Trace-Id";

#[derive(PartialEq, Debug)]
pub enum SamplingDecision {
    /// Sampled indicates the current segment has been
    /// sampled and will be sent to the X-Ray daemon.
    Sampled,
    /// NotSampled indicates the current segment has
    /// not been sampled.
    NotSampled,
    ///sampling decision will be
    /// made by the downstream service and propagated
    /// back upstream in the response.
    Requested,
    /// Unknown indicates no sampling decision will be made.
    Unknown,
}

impl Default for SamplingDecision {
    fn default() -> Self {
        SamplingDecision::Unknown
    }
}

/// Parsed representation of `X-Amzn-Trace-Id` request header
#[derive(PartialEq, Debug, Default)]
pub struct Header {
    trace_id: String,
    parent_id: Option<String>,
    sampling_decision: SamplingDecision,
    additional_data: Option<HashMap<String, String>>,
}

impl FromStr for Header {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(';')
            .try_fold(Header::default(), |mut header, line| {
                let pos = line
                    .find('=')
                    .ok_or_else(|| format!("invalid key=value: no `=` found in `{}`", s))?;
                let (key, value) = (&line[..pos], &line[pos + 1..]);
                match key {
                    "Root" => header.trace_id = value.into(),
                    "Parent" => header.parent_id = Some(value.into()),
                    "Sampled" => {
                        header.sampling_decision = match value {
                            "1" => SamplingDecision::Sampled,
                            "0" => SamplingDecision::NotSampled,
                            "?" => SamplingDecision::Requested,
                            _ => SamplingDecision::Unknown,
                        }
                    }
                    "Self" => {
                        let mut data = HashMap::new();
                        data.insert("Self".into(), value.into());
                        header.additional_data = Some(data)
                    }
                    _ => (),
                }
                Ok(header)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_with_parent_from_str() {
        assert_eq!(
            "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=1"
                .parse::<Header>(),
            Ok(Header {
                trace_id: "1-5759e988-bd862e3fe1be46a994272793".into(),
                parent_id: Some("53995c3f42cd8ad8".into()),
                sampling_decision: SamplingDecision::Sampled,
                ..Header::default()
            })
        )
    }
    #[test]
    fn parse_no_parent_from_str() {
        assert_eq!(
            "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1".parse::<Header>(),
            Ok(Header {
                trace_id: "1-5759e988-bd862e3fe1be46a994272793".into(),
                parent_id: None,
                sampling_decision: SamplingDecision::Sampled,
                ..Header::default()
            })
        )
    }
}
