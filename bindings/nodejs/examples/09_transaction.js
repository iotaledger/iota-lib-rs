// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// In this example we will send a transaction
async function run() {
    const { Client } = require('@iota/client');

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

    try {
        // We generate an address from our seed so that we send the funds to ourselves
        const addresses = await client.generateAddresses(signer, {
            range: {
                start: 1,
                end: 2,
            },
        });

        // Insert the output address and amount to spend. The amount cannot be zero.
        const message = await client.generateMessage(signer, {
            output: { address: addresses[0], amount: 1000000 },
        });
        console.log('Message: ', message, '\n');

        // Send transaction
        const messageId = await client.postMessage(message);

        // TODO: link doesn't work (Not found), same goes for the rust example (06_simple_message.rs)
        console.log(
            `Transaction sent: https://explorer.iota.org/devnet/message/${messageId}`,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
}

run().then(() => process.exit());
