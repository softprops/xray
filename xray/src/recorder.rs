use crate::{Client, Header, Segment, SegmentId, Subsegment, TraceId};
use serde::Serialize;
use std::{marker::PhantomData, mem, sync::Arc};
use thread_local_object::ThreadLocal;

#[derive(Clone, Default, Debug)]
pub struct Context {
    trace_id: TraceId,
    parent_id: Option<SegmentId>,
    segment_id: SegmentId,
}

struct Inner {
    current: ThreadLocal<Context>,
    client: Client,
}

/// Represents the current state of a (sub)segment context
/// for the current thread
///
pub struct Current {
    recorder: Recorder,
    prev: Option<Context>,
    // make sure this type is !Send since it pokes at thread locals
    _p: PhantomData<*const ()>,
}

unsafe impl Sync for Current {}

impl Drop for Current {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(prev) => {
                self.recorder.0.current.set(prev);
            }
            None => {
                self.recorder.0.current.remove();
            }
        }
    }
}

/// An open trace subsegment
///
/// When dropped, the segment will be recorded
pub struct OpenSubsegment {
    current: Current,
    context: Context,
    state: Option<Subsegment>,
}

impl OpenSubsegment {
    fn new<N>(
        current: Current,
        context: Context,
        name: N,
    ) -> Self
    where
        N: Into<String>,
    {
        let subseg = Subsegment::begin(
            name,
            context.segment_id.clone(),
            context.parent_id.clone(),
            context.trace_id.clone(),
        );

        Self {
            current,
            context,
            state: Some(subseg),
        }
    }

    pub fn subsegment(&mut self) -> &mut Option<Subsegment> {
        &mut self.state
    }
}

// recipie for emiting should be
// if end of last subseg, emit parent + subseg
// if not lastsubset and parent > 100 subseg
//  for each subseg ss
///    if ss.in progress or its subsegs arent help stream them
///    emit subseg and remove from parent
impl Drop for OpenSubsegment {
    fn drop(&mut self) {
        if let Some(mut subsegment) = mem::replace(&mut self.state, None) {
            subsegment.end();
            self.current.recorder.emit(&subsegment);
        }
    }
}

/// An open trace subsegment
///
/// When dropped, the segment will be recorded
pub struct OpenSegment {
    current: Current,
    context: Context,
    state: Option<Segment>,
}

impl OpenSegment {
    fn new(
        current: Current,
        context: Context,
        name: String,
    ) -> Self {
        let segment = Segment::begin(
            name,
            context.segment_id.clone(),
            context.parent_id.clone(),
            context.trace_id.clone(),
        );

        Self {
            current,
            context,
            state: Some(segment),
        }
    }
}

impl Drop for OpenSegment {
    fn drop(&mut self) {
        if let Some(mut segment) = mem::replace(&mut self.state, None) {
            segment.end();
            self.current.recorder.emit(&segment);
        }
    }
}

/// A recorder manages the state of a
/// segment and its corresponding subsegments,
/// recording them when appropriate
#[derive(Clone)]
pub struct Recorder(Arc<Inner>);

impl Default for Recorder {
    fn default() -> Self {
        Self(Arc::new(Inner {
            current: ThreadLocal::new(),
            client: Client::default(),
        }))
    }
}

impl Recorder {
    fn emit<S>(
        &self,
        s: &S,
    ) where
        S: Serialize,
    {
        if let Err(e) = self.0.client.send(&s) {
            log::debug!("error emitting data {:?}", e);
        }
    }
    /// Intended to be used when weaving context through
    /// thread contexts. When dropped, the context will be placed
    /// in its previous state
    pub fn set(
        &self,
        ctx: Context,
    ) -> Current {
        Current {
            recorder: self.clone(),
            prev: self.0.current.set(ctx),
            _p: PhantomData,
        }
    }

    /// Return the current threads current state associated with a trace
    pub fn current(&self) -> Option<Context> {
        self.0.current.get_cloned()
    }

    /// Begins a new trace
    pub fn begin_segment<N>(
        &self,
        name: N,
    ) -> OpenSegment
    where
        N: Into<String>,
    {
        let name = name.into();
        if let Some(current) = self.current() {
            log::debug!(
          "Beginning new segment while another segment exists in the segment context. Overwriting current segment '{}' to start new segment named '{}'.",
          current.segment_id, name
        )
        }
        let trace_id = TraceId::new();
        let segment_id = SegmentId::new();
        let context = Context {
            trace_id,
            segment_id,
            ..Context::default()
        };

        let current = self.set(context.clone());
        OpenSegment::new(current, context, name)
    }

    /// begin a new subsegment which may be the child of another
    /// lambda - (immutable parent) https://github.com/aws/aws-xray-sdk-java/blob/3e0b21c5bafec8d0577768cdfc31f4139c4fbecc/aws-xray-recorder-sdk-core/src/main/java/com/amazonaws/xray/contexts/LambdaSegmentContext.java#L36
    /// thread local - https://github.com/aws/aws-xray-sdk-java/blob/3e0b21c5bafec8d0577768cdfc31f4139c4fbecc/aws-xray-recorder-sdk-core/src/main/java/com/amazonaws/xray/contexts/ThreadLocalSegmentContext.java#L20
    pub fn begin_subsegment<N>(
        &self,
        name: N,
    ) -> OpenSubsegment
    where
        N: Into<String>,
    {
        let context = match self.current() {
            Some(Context {
                trace_id,
                segment_id,
                ..
            }) => Context {
                trace_id,
                parent_id: Some(segment_id),
                segment_id: SegmentId::new(),
            },
            _ => match crate::lambda::header() {
                Some(Header {
                    trace_id,
                    parent_id,
                    ..
                }) => Context {
                    trace_id,
                    parent_id,
                    segment_id: SegmentId::new(),
                },
                _ => Context::default(),
            },
        };
        let current = self.set(context.clone());
        OpenSubsegment::new(current, context, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    #[test]
    #[ignore]
    fn test_recorder() {
        let recorder = Recorder::default();
        let a = recorder.begin_segment("test-segment");
        thread::sleep(Duration::from_secs(1));
        let b = recorder.begin_subsegment("subsegment-b");
        thread::sleep(Duration::from_secs(1));
        let c = recorder.begin_subsegment("subsegment-c");
    }
}
