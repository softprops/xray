use rusoto_core::{DefaultCredentialsProvider, Region};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, ListTablesInput};
use tokio::runtime::Runtime;
use xray_rusoto::TracedRequests;

fn main() {
    let mut rt = Runtime::new().expect("failed to initialize runtime");
    let client = DynamoDbClient::new_with(
        TracedRequests::default(),
        DefaultCredentialsProvider::new().expect("failed to initialize credential provider"),
        Region::default(),
    );
    //let client = DynamoDbClient::new(Region::default());
    println!(
        "{:#?}",
        rt.block_on(client.list_tables(ListTablesInput::default()))
    );
}
