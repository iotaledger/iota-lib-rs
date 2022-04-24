import type { GenerateAddressesOptions } from '../client';
import type { GenerateMessageOptions } from '../generateMessageOptions';
import type { Message } from '../message';
import type { QueryParameter } from '../queryParameters';

export interface __GetInfoPayloadMethod__ {
    name: 'GetInfo';
}

export interface __GetInfoPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetInfoPayloadMethod__;
}

export interface __GetOutputPayloadMethod__ {
    name: 'GetOutput';
    data: {
        outputId: string;
    };
}

export interface __GetOutputPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetOutputPayloadMethod__;
}

export interface __GetOutputIdsPayloadMethod__ {
    name: 'OutputIds';
    data: {
        queryParameters: QueryParameter[];
    };
}

export interface __GetOutputIdsPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetOutputPayloadMethod__;
}

export interface __GetOutputsPayloadMethod__ {
    name: 'GetOutputs';
    data: {
        outputIds: string[];
    };
}

export interface __GetOutputsPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetOutputsPayloadMethod__;
}

export interface __GenerateMnemonicPayloadMethod__ {
    name: 'GenerateMnemonic';
}

export interface __GenerateMnemonicPayload__ {
    cmd: 'CallClientMethod';
    payload: __GenerateMnemonicPayloadMethod__;
}

export interface __MnemonicToHexSeedPayloadMethod__ {
    name: 'MnemonicToHexSeed';
    data: {
        mnemonic: string;
    };
}

export interface __MnemonicToHexSeedPayload__ {
    cmd: 'CallClientMethod';
    payload: __MnemonicToHexSeedPayloadMethod__;
}

export interface __GenerateAddressesPayloadMethod__ {
    name: 'GenerateAddresses';
    data: {
        signer: string;
        options: GenerateAddressesOptions;
    };
}

export interface __GenerateAddressesPayload__ {
    cmd: 'CallClientMethod';
    payload: __GenerateAddressesPayloadMethod__;
}

export interface __PostMessagePayloadMethod__ {
    name: 'PostMessage';
    data: {
        message: Message;
    };
}

export interface __PostMessagePayload__ {
    cmd: 'CallClientMethod';
    payload: __PostMessagePayloadMethod__;
}

export interface __GenerateMessagePayloadMethod__ {
    name: 'GenerateMessage';
    data: {
        signer?: string;
        options?: GenerateMessageOptions;
    };
}

export interface __GenerateMessagePayload__ {
    cmd: 'CallClientMethod';
    payload: __GenerateMessagePayloadMethod__;
}
export interface __GetTipsPayloadMethod__ {
    name: 'GetTips';
}

export interface __GetTipsPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetTipsPayloadMethod__;
}

export interface __GetNetworkInfoPayloadMethod__ {
    name: 'GetNetworkInfo';
}

export interface __GetNetworkInfoPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetNetworkInfoPayloadMethod__;
}

export interface __GetMessageDataPayloadMethod__ {
    name: 'GetMessageData';
    data: {
        messageId: string;
    };
}

export interface __GetMessageDataPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetMessageDataPayloadMethod__;
}

export interface __GetMessageMetadataPayloadMethod__ {
    name: 'GetMessageMetadata';
    data: {
        messageId: string;
    };
}

export interface __GetMessageMetadataPayload__ {
    cmd: 'CallClientMethod';
    payload: __GetMessageMetadataPayloadMethod__;
}
