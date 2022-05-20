import type {
    __GetInfoPayloadMethod__,
    __GetOutputIdsPayloadMethod__,
    __GetOutputPayloadMethod__,
    __GetOutputsPayloadMethod__,
    __GenerateMnemonicPayloadMethod__,
    __MnemonicToHexSeedPayloadMethod__,
    __GenerateAddressesPayloadMethod__,
    __PostBlockPayloadMethod__,
    __GenerateBlockPayloadMethod__,
    __GetTipsPayloadMethod__,
    __GetNetworkInfoPayloadMethod__,
    __GetBlockPayloadMethod__,
    __GetBlockMetadataPayloadMethod__,
    __FindInputsPayloadMethod__,
    __FindOutputsPayloadMethod__,
    __PrepareTransactionPayloadMethod__,
    __SignTransactionPayloadMethod__,
    __SubmitPayloadPayloadMethod__,
    __ParseBech32AddressPayloadMethod__,
    __BlockIdPayloadMethod__,
    __GetNodePayloadMethod__,
    __GetNetworkIdPayloadMethod__,
    __GetBech32HrpPayloadMethod__,
    __GetMinPowScorePayloadMethod__,
    __GetTipsIntervalPayloadMethod__,
    __GetLocalPowPayloadMethod__,
    __GetFallbackToLocalPowPayloadMethod__,
    __GetNodeHealthPayloadMethod__,
    __GetHealthPayloadMethod__,
    __GetNodeInfoPayloadMethod__,
    __GetPeersPayloadMethod__,
    __PostBlockJsonPayloadMethod__,
    __GetBlockRawPayloadMethod__,
    __GetBlockChildrenPayloadMethod__,
    __GetReceiptsPayloadMethod__,
    __GetReceiptsMigratedAtPayloadMethod__,
    __GetTreasuryPayloadMethod__,
    __GetIncludedBlockPayloadMethod__,
    __Bech32ToHexPayloadMethod__,
    __HexToBech32PayloadMethod__,
    __HexPublicKeyToBech32AddressPayloadMethod__,
    __IsAddressValidPayloadMethod__,
    __AliasesOutputIdsPayloadMethod__,
    __AliasOutputIdPayloadMethod__,
    __NftsOutputIdsPayloadMethod__,
    __NftOutputIdPayloadMethod__,
    __FoundriesOutputIdsPayloadMethod__,
    __FoundryOutputIdPayloadMethod__,
    __TryGetOutputsPayloadMethod__,
    __FindBlocksPayloadMethod__,
    __RetryPayloadMethod__,
    __RetryUntilIncludedPayloadMethod__,
    __ConsolidateFundsPayloadMethod__,
    __ReattachPayloadMethod__,
    __ReattachUncheckedPayloadMethod__,
    __PromotePayloadMethod__,
    __PromoteUncheckedPayloadMethod__,
    __UnsyncedNodesPayloadMethod__,
    __GetMilestoneByMilestoneIdPayloadMethod__,
    __GetUtxoChangesByMilestoneIdPayloadMethod__,
    __GetMilestoneByMilestoneIndexPayloadMethod__,
    __GetUtxoChangesByMilestoneIndexPayloadMethod__,
    __StoreMnemonicPayloadMethod__,
} from './client';

export type __ClientPayloadMethods__ =
    | __GetInfoPayloadMethod__
    | __GetOutputPayloadMethod__
    | __GetOutputIdsPayloadMethod__
    | __GetOutputsPayloadMethod__
    | __GenerateMnemonicPayloadMethod__
    | __MnemonicToHexSeedPayloadMethod__
    | __GenerateAddressesPayloadMethod__
    | __PostBlockPayloadMethod__
    | __GenerateBlockPayloadMethod__
    | __GetTipsPayloadMethod__
    | __GetNetworkInfoPayloadMethod__
    | __GetBlockPayloadMethod__
    | __GetBlockMetadataPayloadMethod__
    | __FindInputsPayloadMethod__
    | __FindOutputsPayloadMethod__
    | __PrepareTransactionPayloadMethod__
    | __SignTransactionPayloadMethod__
    | __StoreMnemonicPayloadMethod__
    | __SubmitPayloadPayloadMethod__
    | __ParseBech32AddressPayloadMethod__
    | __BlockIdPayloadMethod__
    | __GetNodePayloadMethod__
    | __GetNetworkIdPayloadMethod__
    | __GetBech32HrpPayloadMethod__
    | __GetMinPowScorePayloadMethod__
    | __GetTipsIntervalPayloadMethod__
    | __GetLocalPowPayloadMethod__
    | __GetFallbackToLocalPowPayloadMethod__
    | __GetNodeHealthPayloadMethod__
    | __GetHealthPayloadMethod__
    | __GetNodeInfoPayloadMethod__
    | __GetPeersPayloadMethod__
    | __PostBlockJsonPayloadMethod__
    | __GetBlockRawPayloadMethod__
    | __GetBlockChildrenPayloadMethod__
    | __GetMilestoneByMilestoneIdPayloadMethod__
    | __GetUtxoChangesByMilestoneIdPayloadMethod__
    | __GetMilestoneByMilestoneIndexPayloadMethod__
    | __GetUtxoChangesByMilestoneIndexPayloadMethod__
    | __GetReceiptsPayloadMethod__
    | __GetReceiptsMigratedAtPayloadMethod__
    | __GetTreasuryPayloadMethod__
    | __GetIncludedBlockPayloadMethod__
    | __Bech32ToHexPayloadMethod__
    | __HexToBech32PayloadMethod__
    | __HexPublicKeyToBech32AddressPayloadMethod__
    | __IsAddressValidPayloadMethod__
    | __AliasesOutputIdsPayloadMethod__
    | __AliasOutputIdPayloadMethod__
    | __NftsOutputIdsPayloadMethod__
    | __NftOutputIdPayloadMethod__
    | __FoundriesOutputIdsPayloadMethod__
    | __FoundryOutputIdPayloadMethod__
    | __TryGetOutputsPayloadMethod__
    | __FindBlocksPayloadMethod__
    | __RetryPayloadMethod__
    | __RetryUntilIncludedPayloadMethod__
    | __ConsolidateFundsPayloadMethod__
    | __ReattachPayloadMethod__
    | __ReattachUncheckedPayloadMethod__
    | __PromotePayloadMethod__
    | __PromoteUncheckedPayloadMethod__
    | __UnsyncedNodesPayloadMethod__;

export interface __SendMessagePayload__ {
    cmd: 'CallClientMethod';
    payload: __ClientPayloadMethods__;
}
