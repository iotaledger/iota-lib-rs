// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { Client, initLogger, SHIMMER_TESTNET_BECH32_HRP } from '@iota/client';
import { writeFile } from 'fs/promises';
require('dotenv').config({ path: '../../../.env' });
import path = require('path');

// From examples directory, run with:
// node ./dist/offline_signing/0_address_generation.js

// In this example we will generate addresses offline which will be used later to find inputs
const ADDRESS_FILE_NAME = path.join(
    __dirname,
    '../../offline_signing/addresses.json',
);

async function run() {
    initLogger();

    // client will connect to testnet by default
    const offlineClient = new Client({
        offline: true,
        nodes: [
            {
                // Insert your node URL here.
                url: 'http://localhost:14265/',
                disabled: false,
            },
        ],
        localPow: true,
    });

    try {
        if (!process.env.NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1) {
            throw new Error('.env mnemonic is undefined, see .env.example');
        }

        const secretManager = {
            Mnemonic: process.env.NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1,
        };

        // Generates addresses offline.
        const offlineGeneratedAddresses = await offlineClient.generateAddresses(
            secretManager,
            {
                range: {
                    start: 0,
                    end: 10,
                },
                bech32Hrp: SHIMMER_TESTNET_BECH32_HRP,
            },
        );

        await writeFile(
            ADDRESS_FILE_NAME,
            JSON.stringify(offlineGeneratedAddresses),
        );
    } catch (error) {
        console.error('Error: ', error);
    }
}

run().then(() => process.exit());
