// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::input_selection::new::{Burn, InputSelection, Requirement},
    block::{
        address::Address,
        output::{AliasId, Output},
        protocol::protocol_parameters,
    },
    Error,
};

use crate::input_selection::{
    build_alias_output, build_basic_output, build_input_signing_data_alias_outputs,
    build_input_signing_data_most_basic_outputs, ALIAS_ID_0, ALIAS_ID_2, BECH32_ADDRESS, BECH32_ADDRESS_ALIAS_SENDER,
    BECH32_ADDRESS_ED25519_SENDER, BECH32_ADDRESS_NFT_SENDER,
};

#[test]
fn input_alias_eq_output_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(1_000_000, alias_id_2, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::build(outputs, inputs.clone(), protocol_parameters)
        .finish()
        .select()
        .unwrap();

    assert_eq!(selected.0, inputs);
}

#[test]
fn input_amount_lt_output_amount() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None)];

    assert!(matches!(
        InputSelection::build(outputs, inputs, protocol_parameters)
            .finish()
            .select(),
        Err(Error::NotEnoughBalance {
            found: 1_000_000,
            // Amount we want to send + storage deposit for alias remainder
            required: 2_251_500,
        })
    ))
}

#[test]
fn basic_output_with_alias_input() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 2_251_500)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None)];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select()
        .unwrap();

    // basic output + alias remainder
    assert_eq!(selected.1.len(), 2);
}

#[test]
fn create_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_input_signing_data_most_basic_outputs(vec![(BECH32_ADDRESS, 2_000_000)]);
    let outputs = vec![build_alias_output(1_000_000, alias_id_0, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select()
        .unwrap();

    // One output should be added for the remainder
    assert_eq!(selected.1.len(), 2);
    // Output contains the new minted alias id
    assert!(selected.1.iter().any(|output| {
        if let Output::Alias(alias_output) = output {
            *alias_output.alias_id() == alias_id_0
        } else {
            false
        }
    }));
}

#[test]
fn burn_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 2_000_000)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None)];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .burn(Burn::new().add_alias(alias_id_2))
        .finish()
        .select()
        .unwrap();

    // No remainder
    assert_eq!(selected.1.len(), 1);
    // Output is a basic output
    assert!(matches!(selected.1[0], Output::Basic(_)));
}

#[test]
fn not_enough_storage_deposit_for_remainder() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_001)]);
    let outputs = vec![build_alias_output(1_000_000, alias_id_2, BECH32_ADDRESS, None, None)];

    assert!(matches!(
        InputSelection::build(outputs, inputs, protocol_parameters)
            .finish()
            .select(),
        Err(Error::BlockError(
            iota_types::block::Error::InsufficientStorageDepositAmount {
                amount: 1,
                required: 213000,
            }
        ))
    ))
}

#[test]
fn missing_input_for_alias_output() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_most_basic_outputs(vec![(BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(1_000_000, alias_id_2, BECH32_ADDRESS, None, None)];

    assert!(matches!(
        InputSelection::build(outputs, inputs, protocol_parameters)
            .finish()
            .select(),
        Err(Error::UnfulfillableRequirement(Requirement::Alias(alias_id))) if alias_id == alias_id_2
    ))
}

#[test]
fn missing_ed25519_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        Some(BECH32_ADDRESS_ED25519_SENDER),
        None,
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_ed25519() && sender == Address::try_from_bech32(BECH32_ADDRESS_ED25519_SENDER).unwrap().1
    ))
}

#[test]
fn missing_ed25519_issuer() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_ED25519_SENDER),
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_ed25519() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ED25519_SENDER).unwrap().1
    ))
}

#[test]
fn missing_alias_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        Some(BECH32_ADDRESS_ALIAS_SENDER),
        None,
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_alias() && sender == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_SENDER).unwrap().1
    ))
}

#[test]
fn missing_alias_issuer() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_ALIAS_SENDER),
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_alias() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_SENDER).unwrap().1
    ))
}

#[test]
fn missing_nft_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        Some(BECH32_ADDRESS_NFT_SENDER),
        None,
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_nft() && sender == Address::try_from_bech32(BECH32_ADDRESS_NFT_SENDER).unwrap().1
    ))
}

#[test]
fn missing_nft_issuer() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_input_signing_data_alias_outputs(vec![(alias_id_2, BECH32_ADDRESS, 1_000_000)]);
    let outputs = vec![build_alias_output(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
    )];

    let selected = InputSelection::build(outputs, inputs, protocol_parameters)
        .finish()
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_nft() && issuer == Address::try_from_bech32(BECH32_ADDRESS_NFT_SENDER).unwrap().1
    ))
}
