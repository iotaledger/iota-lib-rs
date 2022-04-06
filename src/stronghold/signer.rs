// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The [Signer] implementation for [StrongholdAdapter].

use super::{
    common::{DERIVE_OUTPUT_RECORD_PATH, RECORD_HINT, SECRET_VAULT_PATH, SEED_RECORD_PATH, STRONGHOLD_FILENAME},
    StrongholdAdapter,
};
use crate::{
    signing::{types::InputSigningData, GenerateAddressMetadata, SignMessageMetadata, Signer, SignerType},
    Result,
};
use async_trait::async_trait;
use bee_message::{
    address::{Address, Ed25519Address},
    signature::{Ed25519Signature, Signature},
    unlock_block::{SignatureUnlockBlock, UnlockBlock},
};
use crypto::hashes::{blake2b::Blake2b256, Digest};
use iota_stronghold::{Location, ProcResult, Procedure, RecordHint, ResultMessage, SLIP10DeriveInput};
use log::warn;
use std::{ops::Range, time::SystemTime};

#[async_trait]
impl Signer for StrongholdAdapter {
    async fn signer_type(&self) -> SignerType {
        SignerType::Stronghold
    }

    async fn signer_init(&mut self, mnemonic: Option<&str>) -> crate::Result<()> {
        if let Some(mnemonic) = mnemonic {
            self.store_mnemonic(mnemonic.to_string()).await?;
        }

        if self.snapshot_path.is_some() {
            self.read_stronghold_snapshot().await?;
        }

        Ok(())
    }

    async fn signer_sync(&mut self) -> crate::Result<()> {
        self.write_stronghold_snapshot().await?;

        // Make a backup by copying the saved snapshot, if a snapshot path is set.
        if let Some(snapshot_path) = &self.snapshot_path {
            let mut from = snapshot_path.clone();
            let mut to = snapshot_path.clone();

            // XXX: we aren't expecting a system time before the epoch; just don't panic here.
            let timestamp_str = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(ts) => ts.as_secs().to_string(),
                Err(err) => format!("-{}", err.duration().as_secs()),
            };

            from.push(STRONGHOLD_FILENAME);
            to.push(format!("{}-backup-{}.stronghold", STRONGHOLD_FILENAME, timestamp_str));

            tokio::fs::copy(from, to).await?;
        }

        Ok(())
    }

    async fn signer_set_password(&mut self, password: &str) {
        self.set_password(password).await;
    }

    async fn signer_clear_password(&mut self) {
        // Unload Stronghold, regardless of whether a snapshot path has been set or not.
        //
        // It doesn't make sense to allow Stronghold to continue to work when we're purging our cached key. However,
        // by design Stronghold _can_ work when it's already loaded into the memory. Here we unload it first to
        // stop it from working, but this also removes all data from the memory if there isn't a snapshot configured.
        if let Err(e) = self.unload_stronghold().await {
            warn!("Failed to unload Stronghold from memory: {}", e);
        }

        self.clear_key().await;
    }

    async fn signer_gen_addrs(
        &self,
        coin_type: u32,
        account_index: u32,
        address_indexes: Range<u32>,
        internal: bool,
        _metadata: GenerateAddressMetadata,
    ) -> Result<Vec<Address>> {
        // Stronghold arguments.
        let seed_location = SLIP10DeriveInput::Seed(Location::Generic {
            vault_path: SECRET_VAULT_PATH.to_vec(),
            record_path: SEED_RECORD_PATH.to_vec(),
        });
        let derive_location = Location::Generic {
            vault_path: SECRET_VAULT_PATH.to_vec(),
            record_path: DERIVE_OUTPUT_RECORD_PATH.to_vec(),
        };
        let hint = RecordHint::new(RECORD_HINT).unwrap();

        // Addresses to return.
        let mut addresses = Vec::new();

        for address_index in address_indexes {
            // Stronghold 0.4.1 is still using an older version of iota-crypto, so we construct a different one here.
            let chain = crypto05::keys::slip10::Chain::from_u32_hardened(vec![
                44u32,
                coin_type,
                account_index,
                internal as u32,
                address_index,
            ]);

            // Derive a SLIP-10 private key in the vault.
            self.slip10_derive(chain, seed_location.clone(), derive_location.clone(), hint)
                .await?;

            // Get the Ed25519 public key from the derived SLIP-10 private key in the vault.
            let public_key = self.ed25519_public_key(derive_location.clone()).await?;

            // Hash the public key to get the address.
            let hash = Blake2b256::digest(&public_key);

            // Convert the hash into [Address].
            let address = Address::Ed25519(Ed25519Address::new(hash.into()));

            // Collect it.
            addresses.push(address)
        }

        Ok(addresses)
    }

    async fn signer_unlock<'a>(
        &self,
        input: &InputSigningData,
        essence_hash: &[u8; 32],
        _: &SignMessageMetadata<'a>,
    ) -> Result<UnlockBlock> {
        // Stronghold arguments.
        let seed_location = SLIP10DeriveInput::Seed(Location::Generic {
            vault_path: SECRET_VAULT_PATH.to_vec(),
            record_path: SEED_RECORD_PATH.to_vec(),
        });
        let derive_location = Location::Generic {
            vault_path: SECRET_VAULT_PATH.to_vec(),
            record_path: DERIVE_OUTPUT_RECORD_PATH.to_vec(),
        };
        let hint = RecordHint::new(RECORD_HINT).unwrap();

        // Stronghold asks for an older version of [Chain], so we have to perform a conversion here.
        let chain = {
            let raw: Vec<u32> = input
                .chain
                .as_ref()
                .unwrap()
                .segments()
                .iter()
                // XXX: "ser32(i)". RTFSC: [crypto::keys::slip10::Segment::from_u32()]
                .map(|seg| u32::from_be_bytes(seg.bs()))
                .collect();

            crypto05::keys::slip10::Chain::from_u32_hardened(raw)
        };

        // Derive a SLIP-10 private key in the vault.
        self.slip10_derive(chain, seed_location.clone(), derive_location.clone(), hint)
            .await?;

        // Get the Ed25519 public key from the derived SLIP-10 private key in the vault.
        let public_key = self.ed25519_public_key(derive_location.clone()).await?;

        // Sign the message with the derived SLIP-10 private key in the vault.
        let signature = self.ed25519_sign(derive_location.clone(), essence_hash).await?;

        // Convert the raw bytes into [UnlockBlock].
        let unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::new(Signature::Ed25519(
            Ed25519Signature::new(public_key, signature),
        )));

        Ok(unlock_block)
    }
}

/// Private methods for the signer implementation.
impl StrongholdAdapter {
    /// Store a mnemonic into the Stronghold vault.
    pub async fn store_mnemonic(&mut self, mnemonic: String) -> Result<()> {
        // Stronghold arguments.
        let output = Location::Generic {
            vault_path: SECRET_VAULT_PATH.to_vec(),
            record_path: SEED_RECORD_PATH.to_vec(),
        };
        let hint = RecordHint::new("wallet.rs-seed").unwrap();

        // Trim the mnemonic, in case it hasn't been, as otherwise the restored seed would be wrong.
        let trimmed_mnemonic = mnemonic.trim().to_string();

        // Check if the mnemonic is valid.
        crypto::keys::bip39::wordlist::verify(&trimmed_mnemonic, &crypto::keys::bip39::wordlist::ENGLISH)
            .map_err(|e| crate::Error::InvalidMnemonic(format!("{:?}", e)))?;

        // Try to load the snapshot to see if we're creating a new Stronghold vault or not.
        //
        // XXX: The current design of [Error] doesn't allow us to see if it's really a "file does
        // not exist" error or not. Better throw errors other than that, but now we just leave it
        // like this, as if so then later operations would throw errors too.
        self.read_stronghold_snapshot().await.unwrap_or(());

        // If the snapshot has successfully been loaded, then we need to check if there has been a
        // mnemonic stored in Stronghold or not to prevent overwriting it.
        if self.snapshot_loaded && self.stronghold.record_exists(output.clone()).await {
            return Err(crate::Error::StrongholdMnemonicAlreadyStored);
        }

        // Execute the BIP-39 recovery procedure to put it into the vault (in memory).
        self.bip39_recover(trimmed_mnemonic, None, output, hint).await?;

        // Persist Stronghold to the disk, if a snapshot path has been set.
        if self.snapshot_path.is_some() {
            self.write_stronghold_snapshot().await?;
        }

        // Now we consider that the snapshot has been loaded; it's just in a reversed order.
        self.snapshot_loaded = true;

        Ok(())
    }

    /// Execute [Procedure::BIP39Recover] in Stronghold to put a mnemonic into the Stronghold vault.
    async fn bip39_recover(
        &self,
        mnemonic: String,
        passphrase: Option<String>,
        output: Location,
        hint: RecordHint,
    ) -> Result<()> {
        match self
            .stronghold
            .runtime_exec(Procedure::BIP39Recover {
                mnemonic,
                passphrase,
                output,
                hint,
            })
            .await
        {
            // BIP-39 recovery success.
            ProcResult::BIP39Recover(ResultMessage::Ok(_)) => Ok(()),
            // BIP-39 recovery failure.
            // XXX: Should we create a separate error type for this error?
            ProcResult::BIP39Recover(ResultMessage::Error(err)) => Err(crate::Error::StrongholdProcedureError(err)),
            // Generic Stronghold procedure failure.
            ProcResult::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
            // Unexpected result type, which should never happen!
            err => {
                warn!(
                    "StrongholdSigner::bip39_recover(): unexpected result from Stronghold: {:?}",
                    err
                );
                Err(crate::Error::StrongholdProcedureError(format!("{:?}", err)))
            }
        }
    }

    /// Execute [Procedure::SLIP10Derive] in Stronghold to derive a SLIP-10 private key in the Stronghold vault.
    async fn slip10_derive(
        &self,
        // Stronghold 0.4.1 is still using an older version of iota-crypto, so we ask for a different one here.
        chain: crypto05::keys::slip10::Chain,
        input: SLIP10DeriveInput,
        output: Location,
        hint: RecordHint,
    ) -> Result<()> {
        match self
            .stronghold
            .runtime_exec(Procedure::SLIP10Derive {
                chain,
                input,
                output,
                hint,
            })
            .await
        {
            // SLIP-10 derivation success.
            // We don't care about the returned value, as later we use the output in vault.
            ProcResult::SLIP10Derive(ResultMessage::Ok(_)) => Ok(()),
            // SLIP-10 derivation failure.
            // XXX: Should we create a separate error type for this error?
            ProcResult::SLIP10Derive(ResultMessage::Error(err)) => Err(crate::Error::StrongholdProcedureError(err)),
            // Generic Stronghold procedure failure.
            ProcResult::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
            // Unexpected result type, which should never happen!
            err => {
                warn!(
                    "StrongholdSigner::slip10_derive(): unexpected result from Stronghold: {:?}",
                    err
                );
                Err(crate::Error::StrongholdProcedureError(format!("{:?}", err)))
            }
        }
    }

    /// Execute [Procedure::Ed25519PublicKey] in Stronghold to get an Ed25519 public key from the SLIP-10 private key
    /// located in `private_key`.
    async fn ed25519_public_key(&self, private_key: Location) -> Result<[u8; 32]> {
        match self
            .stronghold
            .runtime_exec(Procedure::Ed25519PublicKey { private_key })
            .await
        {
            // Ed25519 public key get success.
            ProcResult::Ed25519PublicKey(ResultMessage::Ok(pubkey)) => Ok(pubkey),
            // Ed25519 public key get failure.
            // XXX: Should we create a separate error type for this error?
            ProcResult::Ed25519PublicKey(ResultMessage::Error(err)) => Err(crate::Error::StrongholdProcedureError(err)),
            // Generic Stronghold procedure failure.
            ProcResult::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
            // Unexpected result type, which should never happen!
            err => {
                warn!(
                    "StrongholdSigner::ed25519_public_key(): unexpected result from Stronghold: {:?}",
                    err
                );
                Err(crate::Error::StrongholdProcedureError(format!("{:?}", err)))
            }
        }
    }

    /// Execute [Procedure::Ed25519Sign] in Stronghold to sign `msg` with `private_key` stored in the Stronghold vault.
    async fn ed25519_sign(&self, private_key: Location, msg: &[u8]) -> Result<[u8; 64]> {
        match self
            .stronghold
            .runtime_exec(Procedure::Ed25519Sign {
                private_key,
                msg: msg.to_vec(),
            })
            .await
        {
            // Ed25519 sign success.
            ProcResult::Ed25519Sign(ResultMessage::Ok(msg)) => Ok(msg),
            // Ed25519 sign failure.
            // XXX: Should we create a separate error type for this error?
            ProcResult::Ed25519Sign(ResultMessage::Error(err)) => Err(crate::Error::StrongholdProcedureError(err)),
            // Generic Stronghold procedure failure.
            ProcResult::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
            // Unexpected result type, which should never happen!
            err => {
                warn!(
                    "StrongholdSigner::ed25519_sign(): unexpected result from Stronghold: {:?}",
                    err
                );
                Err(crate::Error::StrongholdProcedureError(format!("{:?}", err)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::IOTA_COIN_TYPE, signing::Network};
    use std::path::Path;

    #[tokio::test]
    async fn test_signer_sync() {
        let snapshot_path = Path::new("stronghold-test");
        let mut signer = StrongholdAdapter::builder()
            .snapshot_path(snapshot_path.to_path_buf())
            .password("drowssap")
            .build();

        let timestamp_str = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(ts) => ts.as_secs().to_string(),
            Err(err) => format!("-{}", err.duration().as_secs()),
        };
        let backup_filename = format!("{}-backup-{}.stronghold", STRONGHOLD_FILENAME, timestamp_str);

        let mut saved_file = snapshot_path.to_path_buf();
        let mut backup_file = snapshot_path.to_path_buf();
        saved_file.push(STRONGHOLD_FILENAME);
        backup_file.push(backup_filename);

        signer.signer_sync().await.unwrap();

        assert!(saved_file.exists());
        assert!(backup_file.exists());

        // Remove artifacts, but don't care about the result.
        tokio::fs::remove_dir_all(snapshot_path).await.unwrap_or(());
    }

    #[tokio::test]
    async fn test_address_generation() {
        let stronghold_path = Path::new("test.stronghold");
        let mnemonic = "giant dynamic museum toddler six deny defense ostrich bomb access mercy blood explain muscle shoot shallow glad autumn author calm heavy hawk abuse rally";

        let mut signer = StrongholdAdapter::builder()
            .snapshot_path(stronghold_path.to_path_buf())
            .password("drowssap")
            .build();

        signer.signer_init(Some(mnemonic)).await.unwrap();

        // The snapshot should have been on the disk now.
        assert!(stronghold_path.exists());

        let addresses = signer
            .signer_gen_addrs(
                IOTA_COIN_TYPE,
                0,
                0..1,
                false,
                GenerateAddressMetadata {
                    syncing: false,
                    network: Network::Testnet,
                },
            )
            .await
            .unwrap();

        assert_eq!(
            addresses[0].to_bech32("atoi"),
            "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string()
        );

        // Remove garbage after test, but don't care about the result
        std::fs::remove_file(stronghold_path).unwrap_or(());
    }
}
