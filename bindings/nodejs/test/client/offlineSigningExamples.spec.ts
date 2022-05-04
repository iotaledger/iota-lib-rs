import { Client, SHIMMER_TESTNET_BECH32_HRP } from '../../lib';
import '../customMatchers';
import 'dotenv/config';
import { addresses } from '../fixtures/addresses';
import * as preparedTransactionJson from '../fixtures/preparedTransaction.json';
import * as signedTransactionJson from '../fixtures/signedTransaction.json';
import type { IPreparedTransactionData } from '../../types';
import type { PayloadTypes } from '@iota/types';

const onlineClient = new Client({
    nodes: [
        {
            // Insert your node URL here.
            url: process.env.NODE_URL || 'http://localhost:14265',
            disabled: false,
        },
    ],
    localPow: true,
});

const offlineClient = new Client({
    offline: true,
    nodes: [
        {
            url: process.env.NODE_URL || 'http://localhost:14265',
            disabled: false,
        },
    ],
    localPow: true,
});

const secretManager = {
    Mnemonic:
        'endorse answer radar about source reunion marriage tag sausage weekend frost daring base attack because joke dream slender leisure group reason prepare broken river',
};

describe('Offline signing examples', () => {
    it('generates addresses offline', async () => {
        const addresses = await offlineClient.generateAddresses(secretManager, {
            range: {
                start: 0,
                end: 10,
            },
            bech32Hrp: SHIMMER_TESTNET_BECH32_HRP,
        });

        expect(addresses.length).toBe(10);
        addresses.forEach((address) => {
            expect(address).toBeValidAddress();
        });
    });

    // transaction tests disabled for workflows, because they fail if we don't have funds
    it.skip('prepares a transaction', async () => {
        const address =
            'rms1qqv5avetndkxzgr3jtrswdtz5ze6mag20s0jdqvzk4fwezve8q9vkpnqlqe';
        const amount = 1000000;

        const inputs = await onlineClient.findInputs(addresses, amount);

        const preparedTransaction = await onlineClient.prepareTransaction(
            undefined,
            {
                inputs,
                output: { address, amount: amount.toString() },
            },
        );

        // TODO: more assertions
        expect(preparedTransaction.essence.type).toBe(1);
    });

    it('signs a transaction', async () => {
        const signedTransaction = await offlineClient.signTransaction(
            secretManager,
            // Imported JSON is typed with literal types
            preparedTransactionJson as unknown as IPreparedTransactionData,
        );

        // TODO: more assertions
        expect(signedTransaction.type).toBe(6);
    });

    // transaction tests disabled for workflows, because they fail if we don't have funds
    it.skip('sends a transaction', async () => {
        // Send message with the signed transaction as a payload
        const message = await onlineClient.submitPayload(
            // Imported JSON is typed with literal types
            signedTransactionJson as unknown as PayloadTypes,
        );

        expect(message.payload).toBeDefined();

        const messageId = await onlineClient.messageId(message);

        // TODO: more assertions
        expect(messageId).toBeValidMessageId;
    });
});
