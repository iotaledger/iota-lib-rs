// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we get inputs and prepare a transaction.
//! `cargo run --example 1_transaction_preparation --release`.

use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    path::Path,
};

use iota_client::{
    api::{PreparedTransactionData, PreparedTransactionDataDto},
    Client, Result,
};

const ADDRESS_FILE_NAME: &str = "examples/offline_signing/addresses.json";
const PREPARED_TRANSACTION_FILE_NAME: &str = "examples/offline_signing/prepared_transaction.json";

#[tokio::main]
async fn main() -> Result<()> {
    // Address to which we want to send the amount.
    let address = "rms1qruzprxum2934lr3p77t96pzlecxv8pjzvtjrzdcgh2f5exa22n6ga0vm69";
    // The amount to send.
    let amount = 1_000_000;

    // Creates a client instance.
    let online_client = Client::builder()
        // Insert your node URL here.
        .with_node("http://localhost:14265")?
        .with_node_sync_disabled()
        .finish()
        .await?;

    // Recovers addresses from example `0_address_generation`.
    let addresses = read_addresses_from_file(ADDRESS_FILE_NAME)?;
    // Gets enough inputs related to these addresses to cover the amount.
    let inputs = online_client.find_inputs(addresses, amount).await?;

    // Prepares the transaction.
    let mut transaction_builder = online_client.block();
    for input in inputs {
        transaction_builder = transaction_builder.with_input(input)?;
    }
    let prepared_transaction = transaction_builder
        .with_output(address, amount)?
        .prepare_transaction()
        .await?;

    println!("Prepared transaction sending {} to {}.", amount, address);

    write_transaction_to_file(PREPARED_TRANSACTION_FILE_NAME, prepared_transaction)
}

fn read_addresses_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut file = File::open(&path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;

    Ok(serde_json::from_str(&json)?)
}

fn write_transaction_to_file<P: AsRef<Path>>(path: P, prepared_transaction: PreparedTransactionData) -> Result<()> {
    let json = serde_json::to_string_pretty(&PreparedTransactionDataDto::from(&prepared_transaction))?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{}", json);

    file.write_all(json.as_bytes())?;

    Ok(())
}
