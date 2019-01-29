// run cargo +nightly bench

#![feature(test)]
extern crate test;

use test::Bencher;
use xray::{SegmentId, TraceId};

#[bench]
fn bench_trace_id(b: &mut Bencher) {
    b.iter(|| TraceId::new())
}

#[bench]
fn bench_trace_id_display(b: &mut Bencher) {
    b.iter(|| format!("{}", TraceId::new()))
}

#[bench]
fn bench_span_id(b: &mut Bencher) {
    b.iter(|| SegmentId::new())
}

#[bench]
fn bench_span_id_display(b: &mut Bencher) {
    b.iter(|| format!("{}", SegmentId::new()))
}
