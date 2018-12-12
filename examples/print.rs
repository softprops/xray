use xray;

fn main() {
    println!("{}", xray::trace_id());
    println!("{}", xray::segment_id());
}
