// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// In this example we will create addresses from a mnemonic defined in .env
async function run() {
    const {
        Client,
        initLogger,
        SHIMMER_TESTNET_BECH32_HRP,
    } = require('@iota/client');

    initLogger({
        color_enabled: true,
        name: './client.log',
        level_filter: 'debug',
    });

    // client will connect to testnet by default
    const client = new Client({
        nodes: [
            {
                url: 'http://localhost:14265',
                auth: null,
                disabled: false,
            },
        ],
        localPow: true,
    });

    require('dotenv').config();
    const signer = JSON.stringify({
        Mnemonic: process.env.NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1,
    });

    const defaultOptions = {};

    const customOptions = {
        accountIndex: 0,
        range: {
            start: 0,
            end: 4,
        },
    };

    const offlineGeneratedOptions = {
        accountIndex: 0,
        range: {
            start: 0,
            end: 4,
        },
        bech32Hrp: SHIMMER_TESTNET_BECH32_HRP,
    };

    try {
        // Generate addresses with default account index and range
        const defaultAddresses = await client.generateAddresses(
            signer,
            defaultOptions,
        );
        console.log(
            'List of generated public addresses:',
            defaultAddresses,
            '\n',
        );

        // Generate addresses with custom account index and range
        const customAddresses = await client.generateAddresses(
            signer,
            customOptions,
        );
        console.log(
            `List of generated public addresses:`,
            customAddresses,
            '\n',
        );

        // TODO: How to implement this? Is a new client_method required?
        // Generate public (false) & internal (true) addresses
        // console.log(
        //     `List of generated public and internal addresses: \n${bech32Addresses}\n`,
        // );

        // Generate public addresses offline with the bech32_hrp defined
        const offlineGeneratedAddresses = await client.generateAddresses(
            signer,
            offlineGeneratedOptions,
        );
        console.log(
            `List of offline generated public addresses:`,
            offlineGeneratedAddresses,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
}

run().then(() => process.exit());
