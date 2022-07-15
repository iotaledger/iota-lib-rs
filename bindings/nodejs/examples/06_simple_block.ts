// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { Client, initLogger } from '@iota/client';
require('dotenv').config({ path: '../.env' });

// Run with command:
// node ./dist/06_simple_block.js

// In this example we will send a block without a payload
async function run() {
    initLogger();
    if (!process.env.NODE_URL) {
        throw new Error('.env NODE_URL is undefined, see .env.example');
    }

    const client = new Client({
        // Insert your node URL in the .env.
        nodes: [process.env.NODE_URL],
        localPow: true,
    });

    try {
        // Create block with no payload
        const block = await client.generateBlock();
        console.log('Block:', block, '\n');

        // Send block
        const blockId = await client.postBlock(block);

        console.log(
            `Empty block sent: ${process.env.EXPLORER_URL}/block/${blockId}`,
        );
    } catch (error) {
        console.error('Error: ', error);
    }
}

run().then(() => process.exit());
