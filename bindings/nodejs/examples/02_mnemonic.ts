// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { Client, CoinType, initLogger } from '@iota/client';

// Run with command:
// node ./dist/02_mnemonic.js

// In this example we will generate a mnemonic and generate the first address with the Shimmer coin type,
// following BIP-0044
async function run() {
    initLogger();

    // client will connect to testnet by default
    const client = new Client({
        nodes: [
            {
                // Insert your node URL here.
                url: 'http://localhost:14265',
            },
        ],
        localPow: true,
    });

    try {
        const mnemonic = await client.generateMnemonic();

        const secretManager = { Mnemonic: mnemonic };

        // Generate addresses with custom account index and range
        const addresses = await client.generateAddresses(secretManager, {
            coinType: CoinType.Shimmer,
            accountIndex: 0,
            range: {
                start: 0,
                end: 1,
            },
        });

        console.log('First public address: ', addresses[0]);
    } catch (error) {
        console.error('Error: ', error);
    }
}

run().then(() => process.exit());