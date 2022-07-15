// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Calls `GET /api/core/v2/milestones/by-index/{index}`.
//! Returns milestone data as raw bytes by its index.
//! Run: `cargo run --example node_api_core_get_milestone_by_index_raw --release -- [NODE URL]`.

use std::env;

use dotenv::dotenv;
use iota_client::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Takes the node URL from command line argument or use one from env as default.
    let node_url = std::env::args().nth(1).unwrap_or_else(|| {
        // This example uses dotenv, which is not safe for use in production.
        dotenv().ok();
        env::var("NODE_URL").unwrap()
    });

    // Creates a client instance with that node.
    let client = Client::builder()
        .with_node(&node_url)?
        .with_node_sync_disabled()
        .finish()
        .await?;

    // Fetches the latest milestone index from the node.
    let info = client.get_info().await?;
    let milestone_index = info.node_info.status.latest_milestone.index;
    // Sends the request.
    let milestone = client.get_milestone_by_index_raw(milestone_index).await?;

    // Prints the response.
    println!("{:?}", milestone);

    Ok(())
}
