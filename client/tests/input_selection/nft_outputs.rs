// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::input_selection::new::{Burn, InputSelection, Requirement},
    block::{
        address::Address,
        output::{NftId, Output},
        protocol::protocol_parameters,
    },
    Error,
};

use crate::input_selection::{
    build_basic_output, build_inputs, build_nft_output, unsorted_eq,
    Build::{Basic, Nft},
    BECH32_ADDRESS, BECH32_ADDRESS_ALIAS_SENDER, BECH32_ADDRESS_ED25519_SENDER, BECH32_ADDRESS_NFT_SENDER, NFT_ID_0,
    NFT_ID_1, NFT_ID_2,
};

#[test]
fn input_nft_eq_output_nft() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs, inputs);
    assert_eq!(selected.outputs, outputs);
}

#[test]
fn input_amount_lt_output_amount() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientBaseTokenAmount {
            found: 1_000_000,
            // Amount we want to send + storage deposit for nft remainder
            required: 2_229_500,
        })
    ));
}

#[test]
fn basic_output_with_nft_input() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(2_229_500, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters)
        .select()
        .unwrap();

    // basic output + nft remainder
    assert_eq!(selected.outputs.len(), 2);
}

#[test]
fn mint_nft() {
    let protocol_parameters = protocol_parameters();
    let nft_id_0 = NftId::from_str(NFT_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(2_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_0, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters)
        .select()
        .unwrap();

    // One output should be added for the remainder
    assert_eq!(selected.outputs.len(), 2);
    // Output contains the new minted nft id
    assert!(selected.outputs.iter().any(|output| {
        if let Output::Nft(nft_output) = output {
            *nft_output.nft_id() == nft_id_0
        } else {
            false
        }
    }));
}

#[test]
fn burn_nft() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(2_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_basic_output(2_000_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters)
        .burn(Burn::new().add_nft(nft_id_2))
        .select()
        .unwrap();

    // No remainder
    assert_eq!(selected.outputs.len(), 1);
    // Output is a basic output
    assert!(matches!(selected.outputs[0], Output::Basic(_)));
}

#[test]
fn not_enough_storage_deposit_for_remainder() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_001, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::BlockError(
            iota_types::block::Error::InsufficientStorageDepositAmount {
                amount: 1,
                required: 213000,
            }
        ))
    ));
}

#[test]
fn missing_input_for_nft_output() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Nft(nft_id))) if nft_id == nft_id_2
    ));
}

#[test]
fn missing_input_for_nft_output_but_created() {
    let protocol_parameters = protocol_parameters();
    let nft_id_0 = NftId::from_str(NFT_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_0, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn nft_in_output_and_sender() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Nft(1_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS, None, None),
    ]);
    let mut outputs = vec![build_nft_output(1_000_000, nft_id_1, BECH32_ADDRESS, None, None, None)];
    outputs.push(build_basic_output(
        1_000_000,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
    ));

    let selected = InputSelection::new(inputs.clone(), outputs, protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.iter().any(|output| {
        if let Output::Nft(nft_output) = output {
            *nft_output.nft_id() == nft_id_1
        } else {
            false
        }
    }));
    assert!(selected.outputs.iter().any(|output| output.is_basic()));
}

#[test]
fn missing_ed25519_sender() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_ED25519_SENDER),
        None,
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_ed25519() && sender == Address::try_from_bech32(BECH32_ADDRESS_ED25519_SENDER).unwrap().1
    ));
}

#[test]
fn missing_ed25519_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let nft_id_0 = NftId::from_str(NFT_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_0,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_ED25519_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_ed25519() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ED25519_SENDER).unwrap().1
    ));
}

#[test]
fn missing_ed25519_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_ED25519_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn missing_alias_sender() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_ALIAS_SENDER),
        None,
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_alias() && sender == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_SENDER).unwrap().1
    ));
}

#[test]
fn missing_alias_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let nft_id_0 = NftId::from_str(NFT_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_0,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_ALIAS_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_alias() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_SENDER).unwrap().1
    ));
}

#[test]
fn missing_alias_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_ALIAS_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn missing_nft_sender() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
        None,
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_nft() && sender == Address::try_from_bech32(BECH32_ADDRESS_NFT_SENDER).unwrap().1
    ));
}

#[test]
fn missing_nft_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let nft_id_0 = NftId::from_str(NFT_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_0,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_nft() && issuer == Address::try_from_bech32(BECH32_ADDRESS_NFT_SENDER).unwrap().1
    ));
}

#[test]
fn missing_nft_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let nft_id_2 = NftId::from_str(NFT_ID_2).unwrap();

    let inputs = build_inputs(vec![Nft(1_000_000, nft_id_2, BECH32_ADDRESS, None, None, None)]);
    let outputs = vec![build_nft_output(
        1_000_000,
        nft_id_2,
        BECH32_ADDRESS,
        None,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
    )];

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn increase_nft_amount() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Nft(2_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS, None, None),
    ]);
    let outputs = vec![build_nft_output(3_000_000, nft_id_1, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn decrease_nft_amount() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Nft(2_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS, None, None),
    ]);
    let outputs = vec![build_nft_output(1_000_000, nft_id_1, BECH32_ADDRESS, None, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 1);
    assert_eq!(selected.inputs[0], inputs[0]);
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_basic());
            assert_eq!(output.amount(), 1_000_000);
            assert_eq!(output.as_basic().native_tokens().len(), 0);
            assert_eq!(output.as_basic().unlock_conditions().len(), 1);
            assert_eq!(output.as_basic().features().len(), 0);
            assert_eq!(
                *output.as_basic().address(),
                Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
            );
        }
    });
}

#[test]
fn prefer_basic_to_nft() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Nft(2_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS, None, None),
    ]);
    let outputs = vec![build_basic_output(1_000_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 1);
    assert_eq!(selected.inputs[0], inputs[1]);
    assert_eq!(selected.outputs, outputs);
}

#[test]
fn take_amount_from_nft_to_fund_basic() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Nft(2_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS, None, None),
    ]);
    let outputs = vec![build_basic_output(1_200_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_nft());
            assert_eq!(output.amount(), 1_800_000);
            assert_eq!(output.as_nft().native_tokens().len(), 0);
            assert_eq!(output.as_nft().unlock_conditions().len(), 1);
            assert_eq!(output.as_nft().features().len(), 0);
            assert_eq!(
                *output.as_nft().address(),
                Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
            );
        }
    });
}

#[test]
fn nft_burn_should_validate_nft_sender() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS, None, None),
        Nft(1_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
    ]);
    let outputs = vec![build_basic_output(
        3_000_000,
        BECH32_ADDRESS,
        None,
        Some(BECH32_ADDRESS_NFT_SENDER),
    )];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .burn(Burn::new().add_nft(nft_id_1))
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn nft_burn_should_validate_nft_address() {
    let protocol_parameters = protocol_parameters();
    let nft_id_1 = NftId::from_str(NFT_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS_NFT_SENDER, None, None),
        Nft(1_000_000, nft_id_1, BECH32_ADDRESS, None, None, None),
    ]);
    let outputs = vec![build_basic_output(3_000_000, BECH32_ADDRESS, None, None)];

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .burn(Burn::new().add_nft(nft_id_1))
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}
