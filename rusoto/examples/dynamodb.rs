use rusoto_core::request::HttpClient;
use rusoto_core::DefaultCredentialsProvider;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use xray_rusoto::TracedRequests;

fn main() {
    let client = DynamoDbClient::new_with(
        TracedRequests::new(HttpClient::new().unwrap()),
        DefaultCredentialsProvider::new().unwrap(),
        Default::default(),
    );
    println!("{:#?}", client.list_tables(Default::default()).sync());
}
