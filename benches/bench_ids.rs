// run cargo +nightly bench

#![feature(test)]
extern crate test;

use test::Bencher;
use xray::{segment_id, trace_id};

#[bench]
fn bench_trace_id(b: &mut Bencher) {
    b.iter(|| trace_id())
}

#[bench]
fn bench_span_id(b: &mut Bencher) {
    b.iter(|| segment_id())
}
