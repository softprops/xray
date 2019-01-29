use xray;

fn main() {
    let _client = xray::Client::default();
    println!("{}", xray::TraceId::new());
    println!("{}", xray::SegmentId::new());
}
