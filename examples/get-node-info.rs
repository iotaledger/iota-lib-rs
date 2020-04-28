//! Get node information from an IOTA node.
//!
//! Run with:
//!
//! ```
//! cargo run --example get-node-info
//! ```
use anyhow::Result;
use iota_client;

#[smol_potat::main]
async fn main() -> Result<()> {
    let iota = iota_client::Client::new("https://nodes.comnet.thetangle.org");
    let node_info = iota.get_node_info().await?;
    println!("{:#?}", node_info);
    Ok(())
}
