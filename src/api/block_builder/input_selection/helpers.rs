// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Helper functions used in the input selection

use bee_block::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    output::{
        unlock_condition::{
            AddressUnlockCondition, GovernorAddressUnlockCondition, ImmutableAliasAddressUnlockCondition,
            StateControllerAddressUnlockCondition, StorageDepositReturnUnlockCondition,
        },
        BasicOutputBuilder, NativeTokens, Output, Rent, RentStructure, UnlockCondition,
    },
};

use crate::{secret::types::InputSigningData, Result};

/// Computes the minimum amount that an output needs to have, when native tokens are sent with [AddressUnlockCondition].
pub fn minimum_storage_deposit(
    config: &RentStructure,
    address: &Address,
    native_tokens: &Option<NativeTokens>,
) -> Result<u64> {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    // todo: use `OutputAmount::MIN` when public, see https://github.com/iotaledger/bee/issues/1238
    let mut basic_output_builder = BasicOutputBuilder::new_with_amount(1_000_000_000)?;
    if let Some(native_tokens) = native_tokens {
        basic_output_builder = basic_output_builder.with_native_tokens(native_tokens.clone());
    }
    let basic_output = basic_output_builder
        .add_unlock_condition(address_condition)
        .finish_output()?;

    Ok(basic_output.rent_cost(config))
}

/// Get the `StorageDepositReturnUnlockCondition`, if not expired
pub(crate) fn sdr_not_expired(output: &Output, current_time: u32) -> Option<&StorageDepositReturnUnlockCondition> {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(sdr) = unlock_conditions.storage_deposit_return() {
            let expired = if let Some(expiration) = unlock_conditions.expiration() {
                current_time >= expiration.timestamp()
            } else {
                false
            };

            // We only have to send the storage deposit return back if the output is not expired
            if !expired { Some(sdr) } else { None }
        } else {
            None
        }
    } else {
        None
    }
}

// Inputs need to be sorted before signing, because the reference unlock conditions can only reference a lower index
pub(crate) fn sort_input_signing_data(inputs: Vec<InputSigningData>) -> crate::Result<Vec<InputSigningData>> {
    // filter for ed25519 address first, safe to unwrap since we encoded it before
    let mut sorted_inputs = inputs
        .clone()
        .into_iter()
        .filter(|input| Address::try_from_bech32(&input.bech32_address).unwrap().1.kind() == Ed25519Address::KIND)
        .collect::<Vec<InputSigningData>>();

    for input in &inputs {
        // Don't add outputs duplicated
        if sorted_inputs
            .iter()
            .any(|i| i.output_id().unwrap() == input.output_id().unwrap())
        {
            continue;
        }

        match sorted_inputs.iter().position(|input_signing_data| {
            match Address::try_from_bech32(&input.bech32_address) {
                Ok((_, unlock_address)) => match unlock_address {
                    Address::Alias(unlock_address) => {
                        if let Output::Alias(alias_output) = &input_signing_data.output {
                            *unlock_address.alias_id()
                                == alias_output
                                    .alias_id()
                                    .or_from_output_id(input_signing_data.output_id().expect("Invalid output id"))
                        } else {
                            false
                        }
                    }
                    Address::Nft(unlock_address) => {
                        if let Output::Nft(nft_output) = &input_signing_data.output {
                            *unlock_address.nft_id()
                                == nft_output
                                    .nft_id()
                                    .or_from_output_id(input_signing_data.output_id().expect("Invalid output id"))
                        } else {
                            false
                        }
                    }
                    _ => false,
                },
                _ => false,
            }
        }) {
            Some(position) => {
                // Insert after the output we need
                sorted_inputs.insert(position + 1, input.clone());
            }
            None => {
                // insert before address
                let alias_or_nft_address = match &input.output {
                    Output::Alias(alias_output) => Some(Address::Alias(AliasAddress::new(
                        alias_output
                            .alias_id()
                            .or_from_output_id(input.output_id().expect("Invalid output id")),
                    ))),
                    Output::Nft(nft_output) => Some(Address::Nft(NftAddress::new(
                        nft_output
                            .nft_id()
                            .or_from_output_id(input.output_id().expect("Invalid output id")),
                    ))),
                    _ => None,
                };

                if let Some(alias_or_nft_address) = alias_or_nft_address {
                    // Check for existing outputs for this address, and insert before
                    match sorted_inputs.iter().position(|input_signing_data| {
                        Address::try_from_bech32(&input_signing_data.bech32_address)
                            .expect("Safe to unwrap, we encoded it before")
                            .1
                            == alias_or_nft_address
                    }) {
                        Some(position) => {
                            // Insert before the output with this address required for unlocking
                            sorted_inputs.insert(position, input.clone());
                        }
                        // just push output
                        None => sorted_inputs.push(input.clone()),
                    }
                } else {
                    // just push basic or foundry output
                    sorted_inputs.push(input.clone());
                }
            }
        }
    }

    Ok(sorted_inputs)
}

// Check if an address is required for unlockig an output in any unlock condition
pub(crate) fn output_contains_address(output: &Output, address: &Address, current_time: u32) -> bool {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(UnlockCondition::Address(address_unlock_condition)) =
            unlock_conditions.get(AddressUnlockCondition::KIND)
        {
            if address == unlock_conditions.locked_address(address_unlock_condition.address(), current_time) {
                return true;
            }
        }
        if let Some(UnlockCondition::StateControllerAddress(state_controller_unlock_condition)) =
            unlock_conditions.get(StateControllerAddressUnlockCondition::KIND)
        {
            if address == state_controller_unlock_condition.address() {
                return true;
            }
        }
        if let Some(UnlockCondition::GovernorAddress(governor_controller_unlock_condition)) =
            unlock_conditions.get(GovernorAddressUnlockCondition::KIND)
        {
            if address == governor_controller_unlock_condition.address() {
                return true;
            }
        }
        if let Some(UnlockCondition::ImmutableAliasAddress(immutable_alias_address_unlock_condition)) =
            unlock_conditions.get(ImmutableAliasAddressUnlockCondition::KIND)
        {
            if address == immutable_alias_address_unlock_condition.address() {
                return true;
            }
        }
    }

    false
}
