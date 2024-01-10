# Celestia RPC

A collection of traits for interacting with Celestia data availability nodes RPC.

This crate builds on top of the [`jsonrpsee`](https://docs.rs/jsonrpsee) clients.

```rust,no_run
use celestia_rpc::{BlobClient, Client};
use celestia_types::{Blob, nmt::Namespace};
use celestia_types::blob::SubmitOptions;

async fn submit_blob() {
    // create a client to the celestia node
    let token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&token))
        .await
        .expect("Failed creating rpc client");

    // create a blob that you want to submit
    let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    let blob = Blob::new(my_namespace, b"some data to store on blockchain".to_vec())
        .expect("Failed to create a blob");

    // submit it
    client.blob_submit(&[blob], SubmitOptions::default())
        .await
        .expect("Failed submitting the blob");
}
```
