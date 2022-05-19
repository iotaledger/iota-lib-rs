import type {
    IBlock,
    IOutputResponse,
    ITaggedDataPayload,
} from '@iota/types';
import { Client, utf8ToBytes, utf8ToHex } from '../../lib';
import '../customMatchers';
import 'dotenv/config';
import * as addressOutputs from '../fixtures/addressOutputs.json';

const client = new Client({
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

// Skip for CI
describe.skip('Main examples', () => {
    it('gets info about the node', async () => {
        const info = await client.getInfo();

        expect(info.nodeinfo.protocol.bech32HRP).toBe('rms');
        expect(info.nodeinfo.protocol.minPoWScore).toBe(1000);
    });

    it('generates a mnemonic', async () => {
        const mnemonic = await client.generateMnemonic();

        expect(mnemonic).toBeDefined();
    });

    it('generates addresses', async () => {
        const addresses = await client.generateAddresses(secretManager, {
            accountIndex: 0,
            range: {
                start: 0,
                end: 5,
            },
            bech32Hrp: 'rms',
        });

        expect(addresses.length).toBe(5);

        addresses.forEach((address) => {
            expect(address).toBeValidAddress();
        });
    });

    it('gets address outputs', async () => {
        const outputIds = await client.outputIds([
            {
                address:
                    'rms1qpllaj0pyveqfkwxmnngz2c488hfdtmfrj3wfkgxtk4gtyrax0jaxzt70zy',
            },
            { hasExpirationCondition: false },
            { hasTimelockCondition: false },
            { hasStorageDepositReturnCondition: false },
        ]);
        outputIds.forEach((id) => expect(id).toBeValidOutputId());

        const addressOutputs = await client.getOutputs(outputIds);

        expect(addressOutputs).toBeDefined();

        addressOutputs.forEach((output) => {
            expect(output.blockId).toBeValidBlockId();
        });
    });

    it('gets the output of a known output ID', async () => {
        const output = await client.getOutput(
            '0xee8255ece109f4d460fa85d34f2a5f152014633db571220c84d6ebb944f129c00000',
        );

        expect(output.blockId).toBeValidBlockId();
    });

    it('gets the balance of an address', async () => {
        // Generate the first address
        const addresses = await client.generateAddresses(secretManager, {
            accountIndex: 0,
            range: {
                start: 0,
                end: 1,
            },
        });
        expect(addresses[0]).toBeValidAddress();

        // Get output ids of outputs that can be controlled by this address without further unlock constraints
        const outputIds = await client.outputIds([
            { address: addresses[0] },
            { hasExpirationCondition: false },
            { hasTimelockCondition: false },
            { hasStorageDepositReturnCondition: false },
        ]);
        outputIds.forEach((id) => expect(id).toBeValidOutputId());

        // Get outputs by their IDs
        const addressOutputs = await client.getOutputs(outputIds);
        expect(addressOutputs).toBeDefined();
    });

    it('calculates the balance of an address', () => {
        const testOutputs = addressOutputs as IOutputResponse[];

        // Calculate the total amount and native tokens
        let totalAmount = 0;
        const totalNativeTokens: { [id: string]: number } = {};
        for (const outputResponse of testOutputs) {
            const output = outputResponse['output'];

            if ('nativeTokens' in output) {
                output.nativeTokens.forEach(
                    (token) =>
                        (totalNativeTokens[token.id] =
                            (totalNativeTokens[token.id] || 0) +
                            parseInt(token.amount)),
                );
            }

            totalAmount += parseInt(output.amount);
        }

        expect(totalAmount).toBe(1960954000);
        expect(Object.keys(totalNativeTokens).length).toBe(2);
        expect(
            Object.values(totalNativeTokens).reduce(
                (acc: number, val: number) => acc + val,
            ),
        ).toBe(200);
    });

    it('sends a block', async () => {
        const block = await client.generateBlock();

        const blockId = await client.postBlock(block);

        expect(blockId).toBeValidBlockId();
    });

    it('gets block data', async () => {
        const block = await client.generateBlock();

        // Send block
        const blockId = await client.postBlock(block);

        const blockData = await client.getBlockData(blockId);
        const blockMetadata = await client.getBlockMetadata(blockId);

        expect(blockData).toStrictEqual<IBlock>(block);
        expect(blockMetadata.blockId).toBeValidBlockId();
    });

    it('sends a block with a tagged data payload', async () => {
        const block = await client.generateBlock(secretManager, {
            tag: utf8ToBytes('Hello'),
            data: utf8ToBytes('Tangle'),
        });

        // Send block
        const blockId = await client.postBlock(block);

        const fetchedBlock = await client.getBlockData(blockId);

        expect(fetchedBlock.payload).toStrictEqual<ITaggedDataPayload>({
            type: 5,
            tag: utf8ToHex('Hello'),
            data: utf8ToHex('Tangle'),
        });
    });

    it('sends a transaction', async () => {
        const addresses = await client.generateAddresses(secretManager, {
            range: {
                start: 1,
                end: 2,
            },
        });

        const block = await client.generateBlock(secretManager, {
            output: {
                address: addresses[0],
                amount: '1000000',
            },
        });

        // Send transaction
        const blockId = await client.postBlock(block);

        expect(blockId).toBeValidBlockId();
    });
});
