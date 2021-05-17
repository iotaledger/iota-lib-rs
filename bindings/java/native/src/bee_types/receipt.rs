// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::{CopyGetters, Getters};
use std::fmt::{Display, Formatter};

use crate::classes::address::AddressDto;
use bee_rest_api::types::dtos::{
    MigratedFundsEntryDto as RustMigratedFundsEntryDto, PayloadDto as RustPayloadDto, ReceiptDto as RustReceiptDto,
    ReceiptPayloadDto as RustReceiptPayloadDto,
};

#[derive(Getters, CopyGetters)]
pub struct ReceiptDto {
    pub receipt: ReceiptPayloadDto,
    #[getset(get_copy = "pub")]
    pub milestone_index: u32,
}

impl From<RustReceiptDto> for ReceiptDto {
    fn from(receipt: RustReceiptDto) -> Self {
        Self {
            receipt: receipt.receipt.into(),
            milestone_index: receipt.milestone_index.clone(),
        }
    }
}

impl Display for ReceiptDto {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}: {})", self.milestone_index, self.receipt.to_string())
    }
}

#[derive(Getters, CopyGetters)]
pub struct ReceiptPayloadDto {
    #[getset(get_copy = "pub")]
    pub kind: u32,
    #[getset(get_copy = "pub")]
    pub migrated_at: u32,
    pub funds: Vec<MigratedFundsEntryDto>,

    // Actual payload
    pub transaction: RustPayloadDto,

    #[getset(get_copy = "pub")]
    pub last: bool,
}

impl ReceiptPayloadDto {
    pub fn funds(&self) -> Vec<MigratedFundsEntryDto> {
        self.funds.clone()
    }
}

impl Display for ReceiptPayloadDto {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "(kind: {}, migrated at: {}, last: {}, funds: {:?})",
            self.kind, self.migrated_at, self.last, self.funds
        )
    }
}

impl From<RustReceiptPayloadDto> for ReceiptPayloadDto {
    fn from(payload: RustReceiptPayloadDto) -> Self {
        Self {
            kind: payload.kind.clone(),
            migrated_at: payload.migrated_at.clone(),

            // TUrn into index
            funds: payload.funds.into_iter().map(|m| m.into()).collect(),
            transaction: payload.transaction.into(),
            last: payload.last.clone(),
        }
    }
}

#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct MigratedFundsEntryDto {
    #[getset(get = "pub")]
    pub tail_transaction_hash: String,
    pub address: AddressDto,
    #[getset(get_copy = "pub")]
    pub deposit: u64,
}

impl MigratedFundsEntryDto {
    pub fn address(&self) -> AddressDto {
        self.address.clone()
    }
}

impl From<RustMigratedFundsEntryDto> for MigratedFundsEntryDto {
    fn from(value: RustMigratedFundsEntryDto) -> Self {
        MigratedFundsEntryDto {
            tail_transaction_hash: value.tail_transaction_hash,
            address: value.address.into(),
            deposit: value.deposit,
        }
    }
}

// use iota_wallet::message::{
// MessageMigratedFundsEntry as MigratedFundsEntryRust, MessageReceiptPayload as ReceiptPayloadRust,
// TransactionSignatureLockedSingleOutput,
// };
//
// pub struct ReceiptPayload {
// payload: ReceiptPayloadRust,
// }
//
// impl From<ReceiptPayloadRust> for ReceiptPayload {
// fn from(payload: ReceiptPayloadRust) -> Self {
// Self { payload }
// }
// }
//
// impl ReceiptPayload {
// pub fn migrated_at(&self) -> u32 {
// self.payload.migrated_at()
// }
//
// pub fn last(&self) -> bool {
// self.payload.last()
// }
//
// pub fn funds(&self) -> Vec<MigratedFundsEntry> {
// self.payload
// .funds()
// .into_iter()
// .map(|m| MigratedFundsEntry { payload: m.clone() })
// .collect()
// }
// }
//
// pub struct MigratedFundsEntry {
// payload: MigratedFundsEntryRust,
// }
//
// impl MigratedFundsEntry {
// pub fn tail_transaction_hash(&self) -> Vec<u8> {
// self.payload.tail_transaction_hash().as_ref().to_vec()
// }
//
// pub fn output(&self) -> TransactionSignatureLockedSingleOutput {
// self.payload.output().clone()
// }
// }
