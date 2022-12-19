// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Single-threaded PoW miner.

use crypto::{
    encoding::ternary::{b1t6, Btrit, T1B1Buf, TritBuf},
    hashes::{
        blake2b::Blake2b256,
        ternary::{
            curl_p::{CurlPBatchHasher, BATCH_SIZE},
            HASH_LENGTH,
        },
        Digest,
    },
};

use super::{Error, LN_3};

// Should take around one second to reach on an average CPU,
// so shouldn't cause a noticeable delay on timeout_in_seconds.
const POW_ROUNDS_BEFORE_INTERVAL_CHECK: usize = 3000;

/// Single-threaded proof-of-work for Wasm.
pub struct SingleThreadedMiner {
    timeout_in_seconds: Option<u64>,
}

/// Builder for [`SingleThreadedMiner`].
#[derive(Default)]
#[must_use]
pub struct SingleThreadedMinerBuilder {
    timeout_in_seconds: Option<u64>,
}

impl SingleThreadedMinerBuilder {
    /// Create a new `SingleThreadedMinerBuilder.
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Aborts and returns a "cancelled" error after the interval elapses, if set.
    /// New parents (tips) should be fetched and proof-of-work re-run afterwards.
    pub fn with_timeout_in_seconds(mut self, timeout_in_seconds: u64) -> Self {
        self.timeout_in_seconds = Some(timeout_in_seconds);
        self
    }

    /// Build the SingleThreadedMiner.
    pub fn finish(self) -> SingleThreadedMiner {
        SingleThreadedMiner {
            timeout_in_seconds: self.timeout_in_seconds,
        }
    }
}

impl SingleThreadedMiner {
    /// Mine a nonce for provided bytes.
    pub fn nonce(&self, bytes: &[u8], target_score: u32) -> Result<u64, Error> {
        let mut pow_digest = TritBuf::<T1B1Buf>::new();
        let target_zeros =
            (((bytes.len() + std::mem::size_of::<u64>()) as f64 * target_score as f64).ln() / LN_3).ceil() as usize;
        if target_zeros > HASH_LENGTH {
            return Err(Error::InvalidPowScore(target_score, target_zeros));
        }

        let hash = Blake2b256::digest(bytes);
        b1t6::encode::<T1B1Buf>(&hash).iter().for_each(|t| pow_digest.push(t));

        let mut nonce = 0;
        let mut hasher = CurlPBatchHasher::<T1B1Buf>::new(HASH_LENGTH);
        let mut buffers = Vec::<TritBuf<T1B1Buf>>::with_capacity(BATCH_SIZE);
        for _ in 0..BATCH_SIZE {
            let mut buffer = TritBuf::<T1B1Buf>::zeros(HASH_LENGTH);
            buffer[..pow_digest.len()].copy_from(&pow_digest);
            buffers.push(buffer);
        }

        // Counter to reduce number of mining_start.elapsed() calls.
        let mut counter = 0;
        let mining_start = instant::Instant::now();
        loop {
            if let Some(tips_interval) = self.timeout_in_seconds {
                if counter % POW_ROUNDS_BEFORE_INTERVAL_CHECK == 0
                    && mining_start.elapsed() > instant::Duration::from_secs(tips_interval)
                {
                    // Tips interval elapsed, cancel work and get new parents.
                    break;
                }
            }

            for (i, buffer) in buffers.iter_mut().enumerate() {
                let nonce_trits = b1t6::encode::<T1B1Buf>(&(nonce + i as u64).to_le_bytes());
                buffer[pow_digest.len()..pow_digest.len() + nonce_trits.len()].copy_from(&nonce_trits);
                hasher.add(buffer.clone());
            }
            for (i, hash) in hasher.hash().enumerate() {
                let trailing_zeros = hash.iter().rev().take_while(|t| *t == Btrit::Zero).count();
                if trailing_zeros >= target_zeros {
                    return Ok(nonce + i as u64);
                }
            }
            nonce += BATCH_SIZE as u64;
            counter += 1;
        }

        Err(Error::Cancelled)
    }
}
