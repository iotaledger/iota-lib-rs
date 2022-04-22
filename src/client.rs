// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The Client module to connect through HORNET or Bee with API usages

use std::{
    collections::HashSet,
    ops::Range,
    str::FromStr,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bee_message::{
    address::Address,
    input::{Input, UtxoInput, INPUT_COUNT_MAX},
    output::{AliasId, ByteCostConfig, ByteCostConfigBuilder, FoundryId, NftId, OutputId},
    parent::Parents,
    payload::{
        transaction::{TransactionEssence, TransactionId},
        Payload, TaggedDataPayload,
    },
    Message, MessageBuilder, MessageId,
};
use bee_pow::providers::NonceProviderBuilder;
use bee_rest_api::types::{
    dtos::{LedgerInclusionStateDto, PeerDto, ReceiptDto},
    responses::{
        InfoResponse as NodeInfo, MessageMetadataResponse, MilestoneResponse, OutputResponse, TreasuryResponse,
        UtxoChangesResponse as MilestoneUTXOChanges,
    },
};
use crypto::keys::slip10::Seed;
#[cfg(feature = "wasm")]
use gloo_timers::future::TimeoutFuture;
use packable::PackableExt;
use url::Url;
#[cfg(not(feature = "wasm"))]
use {
    crate::api::finish_pow,
    std::collections::HashMap,
    tokio::{
        runtime::Runtime,
        sync::broadcast::{Receiver, Sender},
        time::{sleep, Duration as TokioDuration},
    },
};
#[cfg(feature = "mqtt")]
use {
    crate::node_api::mqtt::{BrokerOptions, MqttEvent, MqttManager, TopicHandlerMap},
    rumqttc::AsyncClient as MqttClient,
    tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender},
};

use crate::{
    api::{
        miner::{ClientMiner, ClientMinerBuilder},
        ClientMessageBuilder, GetAddressesBuilder,
    },
    builder::{ClientBuilder, NetworkInfo},
    constants::{DEFAULT_API_TIMEOUT, DEFAULT_TIPS_INTERVAL, FIVE_MINUTES_IN_SECONDS},
    error::{Error, Result},
    node_api::{high_level::GetAddressBuilder, indexer::query_parameters::QueryParameter},
    node_manager::node::{Node, NodeAuth},
    signing::SignerHandle,
    utils::{
        bech32_to_hex, generate_mnemonic, hash_network, hex_public_key_to_bech32_address, hex_to_bech32,
        is_address_valid, mnemonic_to_hex_seed, mnemonic_to_seed, parse_bech32_address,
    },
};

/// NodeInfo wrapper which contains the nodeinfo and the url from the node (useful when multiple nodes are used)
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfoWrapper {
    /// The returned nodeinfo
    pub nodeinfo: NodeInfo,
    /// The url from the node which returned the nodeinfo
    pub url: String,
}

/// An instance of the client using HORNET or Bee URI
// #[cfg_attr(feature = "wasm", derive(Clone))]
#[derive(Clone)]
pub struct Client {
    #[allow(dead_code)]
    #[cfg(not(feature = "wasm"))]
    pub(crate) runtime: Option<Arc<Runtime>>,
    /// Node manager
    pub(crate) node_manager: crate::node_manager::NodeManager,
    /// Flag to stop the node syncing
    #[cfg(not(feature = "wasm"))]
    pub(crate) sync_kill_sender: Option<Arc<Sender<()>>>,
    /// A MQTT client to subscribe/unsubscribe to topics.
    #[cfg(feature = "mqtt")]
    pub(crate) mqtt_client: Option<MqttClient>,
    #[cfg(feature = "mqtt")]
    pub(crate) mqtt_topic_handlers: Arc<tokio::sync::RwLock<TopicHandlerMap>>,
    #[cfg(feature = "mqtt")]
    pub(crate) broker_options: BrokerOptions,
    #[cfg(feature = "mqtt")]
    pub(crate) mqtt_event_channel: (Arc<WatchSender<MqttEvent>>, WatchReceiver<MqttEvent>),
    pub(crate) network_info: Arc<RwLock<NetworkInfo>>,
    /// HTTP request timeout.
    pub(crate) api_timeout: Duration,
    /// HTTP request timeout for remote PoW API call.
    pub(crate) remote_pow_timeout: Duration,
    #[allow(dead_code)] // not used for wasm
    /// pow_worker_count for local PoW.
    pub(crate) pow_worker_count: Option<usize>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Client");
        d.field("node_manager", &self.node_manager);
        #[cfg(feature = "mqtt")]
        d.field("broker_options", &self.broker_options);
        d.field("network_info", &self.network_info).finish()
    }
}

impl Drop for Client {
    /// Gracefully shutdown the `Client`
    fn drop(&mut self) {
        #[cfg(not(feature = "wasm"))]
        if let Some(sender) = self.sync_kill_sender.take() {
            sender.send(()).expect("failed to stop syncing process");
        }

        #[cfg(not(feature = "wasm"))]
        if let Some(runtime) = self.runtime.take() {
            if let Ok(runtime) = Arc::try_unwrap(runtime) {
                runtime.shutdown_background()
            }
        }

        #[cfg(feature = "mqtt")]
        if let Some(mqtt_client) = self.mqtt_client.take() {
            std::thread::spawn(move || {
                // ignore errors in case the event loop was already dropped
                // .cancel() finishes the event loop right away
                let _ = crate::async_runtime::block_on(mqtt_client.cancel());
            })
            .join()
            .unwrap();
        }
    }
}

impl Client {
    /// Create the builder to instntiate the IOTA Client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Sync the node lists per node_sync_interval milliseconds
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn start_sync_process(
        runtime: &Runtime,
        sync: Arc<RwLock<HashSet<Node>>>,
        nodes: HashSet<Node>,
        node_sync_interval: Duration,
        network_info: Arc<RwLock<NetworkInfo>>,
        mut kill: Receiver<()>,
    ) {
        let node_sync_interval = TokioDuration::from_nanos(
            node_sync_interval
                .as_nanos()
                .try_into()
                .unwrap_or(DEFAULT_TIPS_INTERVAL),
        );

        runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = async {
                            // delay first since the first `sync_nodes` call is made by the builder
                            // to ensure the node list is filled before the client is used
                            sleep(node_sync_interval).await;
                            Client::sync_nodes(&sync, &nodes, &network_info).await;
                    } => {}
                    _ = kill.recv() => {}
                }
            }
        });
    }

    #[cfg(not(feature = "wasm"))]
    pub(crate) async fn sync_nodes(
        sync: &Arc<RwLock<HashSet<Node>>>,
        nodes: &HashSet<Node>,
        network_info: &Arc<RwLock<NetworkInfo>>,
    ) {
        let mut synced_nodes = HashSet::new();
        let mut network_nodes: HashMap<String, Vec<(NodeInfo, Node)>> = HashMap::new();
        for node in nodes {
            // Put the healthy node url into the network_nodes
            if let Ok(info) = Client::get_node_info(&node.url.to_string(), None).await {
                if info.status.is_healthy {
                    match network_nodes.get_mut(&info.protocol.network_name) {
                        Some(network_id_entry) => {
                            network_id_entry.push((info, node.clone()));
                        }
                        None => match &network_info
                            .read()
                            .map_or(NetworkInfo::default().network, |info| info.network.clone())
                        {
                            Some(id) => {
                                if info.protocol.network_name.contains(id) {
                                    network_nodes
                                        .insert(info.protocol.network_name.clone(), vec![(info, node.clone())]);
                                }
                            }
                            None => {
                                network_nodes.insert(info.protocol.network_name.clone(), vec![(info, node.clone())]);
                            }
                        },
                    }
                }
            }
        }
        // Get network_id with the most nodes
        let mut most_nodes = ("network_id", 0);
        for (network_id, node) in network_nodes.iter() {
            if node.len() > most_nodes.1 {
                most_nodes.0 = network_id;
                most_nodes.1 = node.len();
            }
        }
        if let Some(nodes) = network_nodes.get(most_nodes.0) {
            for (info, node_url) in nodes.iter() {
                if let Ok(mut client_network_info) = network_info.write() {
                    client_network_info.network_id = hash_network(&info.protocol.network_name).ok();
                    // todo update protocol version
                    client_network_info.min_pow_score = info.protocol.min_pow_score;
                    client_network_info.bech32_hrp = info.protocol.bech32_hrp.clone();
                    client_network_info.rent_structure = info.protocol.rent_structure.clone();
                    if !client_network_info.local_pow {
                        if info.features.contains(&"PoW".to_string()) {
                            synced_nodes.insert(node_url.clone());
                        }
                    } else {
                        synced_nodes.insert(node_url.clone());
                    }
                }
            }
        }

        // Update the sync list
        if let Ok(mut sync) = sync.write() {
            *sync = synced_nodes;
        }
    }

    /// Get a node candidate from the synced node pool.
    pub async fn get_node(&self) -> Result<Node> {
        if let Some(primary_node) = &self.node_manager.primary_node {
            return Ok(primary_node.clone());
        }
        let pool = self.node_manager.nodes.clone();
        pool.into_iter().next().ok_or(Error::SyncedNodePoolEmpty)
    }

    /// Gets the miner to use based on the PoW setting
    pub async fn get_pow_provider(&self) -> ClientMiner {
        ClientMinerBuilder::new()
            .with_local_pow(self.get_local_pow().await)
            .finish()
    }

    /// Gets the network related information such as network_id and min_pow_score
    /// and if it's the default one, sync it first.
    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let not_synced = self.network_info.read().map_or(true, |info| info.network_id.is_none());

        // For WASM we don't have the node syncing process, which updates the network_info every 60 seconds, but the PoW
        // difficulty or the byte cost could change via a milestone, so we request the nodeinfo every time, so we don't
        // create invalid transactions/messages
        if not_synced || cfg!(feature = "wasm") {
            let info = self.get_info().await?.nodeinfo;
            let network_id = hash_network(&info.protocol.network_name).ok();
            {
                let mut client_network_info = self.network_info.write().map_err(|_| crate::Error::PoisonError)?;
                client_network_info.network_id = network_id;
                client_network_info.min_pow_score = info.protocol.min_pow_score;
                client_network_info.bech32_hrp = info.protocol.bech32_hrp;
            }
        }
        let res = self
            .network_info
            .read()
            .map_or(NetworkInfo::default(), |info| info.clone());
        Ok(res)
    }

    /// Gets the network id of the node we're connecting to.
    pub async fn get_network_id(&self) -> Result<u64> {
        let network_info = self.get_network_info().await?;
        network_info
            .network_id
            .ok_or(Error::MissingParameter("Missing network id."))
    }

    /// returns the bech32_hrp
    pub async fn get_bech32_hrp(&self) -> Result<String> {
        Ok(self.get_network_info().await?.bech32_hrp)
    }

    /// returns the min pow score
    pub async fn get_min_pow_score(&self) -> Result<f64> {
        Ok(self.get_network_info().await?.min_pow_score)
    }

    /// returns the tips interval
    pub async fn get_tips_interval(&self) -> u64 {
        self.network_info
            .read()
            .map_or(DEFAULT_TIPS_INTERVAL, |info| info.tips_interval)
    }

    /// returns if local pow should be used or not
    pub async fn get_local_pow(&self) -> bool {
        self.network_info
            .read()
            .map_or(NetworkInfo::default().local_pow, |info| info.local_pow)
    }

    /// returns the byte cost configuration
    pub async fn get_byte_cost_config(&self) -> Result<ByteCostConfig> {
        let rent_structure = self.get_network_info().await?.rent_structure;
        let byte_cost_config = ByteCostConfigBuilder::new()
            .byte_cost(rent_structure.v_byte_cost)
            .key_factor(rent_structure.v_byte_factor_key)
            .data_factor(rent_structure.v_byte_factor_data)
            .finish();
        Ok(byte_cost_config)
    }

    pub(crate) fn get_timeout(&self) -> Duration {
        self.api_timeout
    }
    pub(crate) fn get_remote_pow_timeout(&self) -> Duration {
        self.remote_pow_timeout
    }

    /// returns the fallback_to_local_pow
    pub async fn get_fallback_to_local_pow(&self) -> bool {
        self.network_info
            .read()
            .map_or(NetworkInfo::default().fallback_to_local_pow, |info| {
                info.fallback_to_local_pow
            })
    }

    /// returns the unsynced nodes.
    #[cfg(not(feature = "wasm"))]
    pub async fn unsynced_nodes(&self) -> HashSet<&Node> {
        self.node_manager.synced_nodes.read().map_or(HashSet::new(), |synced| {
            self.node_manager
                .nodes
                .iter()
                .filter(|node| !synced.contains(node))
                .collect()
        })
    }

    ///////////////////////////////////////////////////////////////////////
    // MQTT API
    //////////////////////////////////////////////////////////////////////

    /// Returns a handle to the MQTT topics manager.
    #[cfg(feature = "mqtt")]
    pub fn subscriber(&mut self) -> MqttManager<'_> {
        MqttManager::new(self)
    }

    /// Returns the mqtt event receiver.
    #[cfg(feature = "mqtt")]
    pub fn mqtt_event_receiver(&self) -> WatchReceiver<MqttEvent> {
        self.mqtt_event_channel.1.clone()
    }

    //////////////////////////////////////////////////////////////////////
    // Node core API
    //////////////////////////////////////////////////////////////////////

    /// GET /health endpoint
    pub async fn get_node_health(url: &str) -> Result<bool> {
        let mut url = Url::parse(url)?;
        url.set_path("health");
        let status = crate::node_manager::http_client::HttpClient::new()
            .get(
                Node {
                    url,
                    auth: None,
                    disabled: false,
                },
                DEFAULT_API_TIMEOUT,
            )
            .await?
            .status();
        match status {
            200 => Ok(true),
            _ => Ok(false),
        }
    }

    /// GET /health endpoint
    pub async fn get_health(&self) -> Result<bool> {
        let mut node = self.get_node().await?;
        node.url.set_path("health");
        let status = self
            .node_manager
            .http_client
            .get(node, DEFAULT_API_TIMEOUT)
            .await?
            .status();
        match status {
            200 => Ok(true),
            _ => Ok(false),
        }
    }

    // todo: only used during syncing, can it be replaced with the other node info function?
    /// GET /api/v2/info endpoint
    pub async fn get_node_info(url: &str, auth: Option<NodeAuth>) -> Result<NodeInfo> {
        let mut url = crate::node_manager::builder::validate_url(Url::parse(url)?)?;
        if let Some(auth) = &auth {
            if let Some((name, password)) = &auth.basic_auth_name_pwd {
                url.set_username(name)
                    .map_err(|_| crate::Error::UrlAuthError("username".to_string()))?;
                url.set_password(Some(password))
                    .map_err(|_| crate::Error::UrlAuthError("password".to_string()))?;
            }
        }
        let path = "api/v2/info";
        url.set_path(path);

        let resp: NodeInfo = crate::node_manager::http_client::HttpClient::new()
            .get(
                Node {
                    url,
                    auth,
                    disabled: false,
                },
                DEFAULT_API_TIMEOUT,
            )
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    /// Returns the node information together with the url of the used node
    /// GET /api/v2/info endpoint
    pub async fn get_info(&self) -> Result<NodeInfoWrapper> {
        crate::node_api::core::routes::get_info(self).await
    }

    /// GET /api/v2/peers endpoint
    pub async fn get_peers(&self) -> Result<Vec<PeerDto>> {
        crate::node_api::core::routes::get_peers(self).await
    }

    /// GET /api/v2/tips endpoint
    pub async fn get_tips(&self) -> Result<Vec<MessageId>> {
        crate::node_api::core::routes::get_tips(self).await
    }

    /// POST /api/v2/messages endpoint
    pub async fn post_message(&self, message: &Message) -> Result<MessageId> {
        crate::node_api::core::routes::post_message(self, message).await
    }

    /// POST JSON to /api/v2/messages endpoint
    pub async fn post_message_json(&self, message: &Message) -> Result<MessageId> {
        crate::node_api::core::routes::post_message_json(self, message).await
    }

    /// GET /api/v2/messages/{messageID} endpoint
    /// Consume the builder and find a message by its identifer. This method returns the given message object.
    pub async fn get_message_data(&self, message_id: &MessageId) -> Result<Message> {
        crate::node_api::core::routes::data(self, message_id).await
    }

    /// GET /api/v2/messages/{messageID}/metadata endpoint
    /// Consume the builder and find a message by its identifer. This method returns the given message metadata.
    pub async fn get_message_metadata(&self, message_id: &MessageId) -> Result<MessageMetadataResponse> {
        crate::node_api::core::routes::metadata(self, message_id).await
    }

    /// GET /api/v2/messages/{messageID}/raw endpoint
    /// Consume the builder and find a message by its identifer. This method returns the given message raw data.
    pub async fn get_message_raw(&self, message_id: &MessageId) -> Result<String> {
        crate::node_api::core::routes::raw(self, message_id).await
    }

    /// GET /api/v2/messages/{messageID}/children endpoint
    /// Consume the builder and returns the list of message IDs that reference a message by its identifier.
    pub async fn get_message_children(&self, message_id: &MessageId) -> Result<Box<[MessageId]>> {
        crate::node_api::core::routes::children(self, message_id).await
    }

    /// GET /api/v2/outputs/{outputId} endpoint
    /// Find an output by its transaction_id and corresponding output_index.
    pub async fn get_output(&self, output_id: &OutputId) -> Result<OutputResponse> {
        crate::node_api::core::routes::get_output(self, output_id).await
    }

    /// GET /api/plugins/indexer/v1/basic-outputs{query} endpoint
    pub fn get_address(&self) -> GetAddressBuilder<'_> {
        GetAddressBuilder::new(self)
    }

    /// GET /api/v2/milestones/{index} endpoint
    /// Get the milestone by the given index.
    pub async fn get_milestone(&self, index: u32) -> Result<MilestoneResponse> {
        crate::node_api::core::routes::get_milestone(self, index).await
    }

    /// GET /api/v2/milestones/{index}/utxo-changes endpoint
    /// Get the milestone by the given index.
    pub async fn get_milestone_utxo_changes(&self, index: u32) -> Result<MilestoneUTXOChanges> {
        crate::node_api::core::routes::get_milestone_utxo_changes(self, index).await
    }

    /// GET /api/v2/receipts endpoint
    /// Get all receipts.
    pub async fn get_receipts(&self) -> Result<Vec<ReceiptDto>> {
        crate::node_api::core::routes::get_receipts(self).await
    }

    /// GET /api/v2/receipts/{migratedAt} endpoint
    /// Get the receipts by the given milestone index.
    pub async fn get_receipts_migrated_at(&self, milestone_index: u32) -> Result<Vec<ReceiptDto>> {
        crate::node_api::core::routes::get_receipts_migrated_at(self, milestone_index).await
    }

    /// GET /api/v2/treasury endpoint
    /// Get the treasury output.
    pub async fn get_treasury(&self) -> Result<TreasuryResponse> {
        crate::node_api::core::routes::get_treasury(self).await
    }

    /// GET /api/v2/transactions/{transactionId}/included-message
    /// Returns the included message of the transaction.
    pub async fn get_included_message(&self, transaction_id: &TransactionId) -> Result<Message> {
        crate::node_api::core::routes::get_included_message(self, transaction_id).await
    }

    //////////////////////////////////////////////////////////////////////
    // Node indexer API
    //////////////////////////////////////////////////////////////////////

    /// api/plugins/indexer/v1/basic-outputs
    pub async fn output_ids(&self, query_parameters: Vec<QueryParameter>) -> Result<Vec<OutputId>> {
        crate::node_api::indexer::routes::output_ids(self, query_parameters).await
    }

    /// api/plugins/indexer/v1/aliases
    pub async fn aliases_output_ids(&self, query_parameters: Vec<QueryParameter>) -> Result<Vec<OutputId>> {
        crate::node_api::indexer::routes::aliases_output_ids(self, query_parameters).await
    }

    /// api/plugins/indexer/v1/aliases/{AliasId}
    pub async fn alias_output_id(&self, alias_id: AliasId) -> Result<OutputId> {
        crate::node_api::indexer::routes::alias_output_id(self, alias_id).await
    }

    /// api/plugins/indexer/v1/nfts
    pub async fn nfts_output_ids(&self, query_parameters: Vec<QueryParameter>) -> Result<Vec<OutputId>> {
        crate::node_api::indexer::routes::nfts_output_ids(self, query_parameters).await
    }

    /// api/plugins/indexer/v1/nfts/{NftId}
    pub async fn nft_output_id(&self, nft_id: NftId) -> Result<OutputId> {
        crate::node_api::indexer::routes::nft_output_id(self, nft_id).await
    }

    /// api/plugins/indexer/v1/foundries
    pub async fn foundries_output_ids(&self, query_parameters: Vec<QueryParameter>) -> Result<Vec<OutputId>> {
        crate::node_api::indexer::routes::foundries_output_ids(self, query_parameters).await
    }

    /// api/plugins/indexer/v1/foundries/{FoundryID}
    pub async fn foundry_output_id(&self, foundry_id: FoundryId) -> Result<OutputId> {
        crate::node_api::indexer::routes::foundry_output_id(self, foundry_id).await
    }

    //////////////////////////////////////////////////////////////////////
    // High level API
    //////////////////////////////////////////////////////////////////////

    /// Get OutputResponse from provided OutputIds (requests are sent in parallel)
    pub async fn get_outputs(&self, output_ids: Vec<OutputId>) -> Result<Vec<OutputResponse>> {
        crate::node_api::core::get_outputs(self, output_ids).await
    }

    /// Try to get OutputResponse from provided OutputIds (requests are sent in parallel and errors are ignored, can be
    /// useful for spent outputs)
    pub async fn try_get_outputs(&self, output_ids: Vec<OutputId>) -> Result<Vec<OutputResponse>> {
        crate::node_api::core::try_get_outputs(self, output_ids).await
    }

    /// Get the inputs of a transaction for the given transaction id.
    pub async fn inputs_from_transaction_id(&self, transaction_id: &TransactionId) -> Result<Vec<OutputResponse>> {
        let message = crate::node_api::core::routes::get_included_message(self, transaction_id).await?;

        let inputs = match message.payload() {
            Some(Payload::Transaction(t)) => match t.essence() {
                TransactionEssence::Regular(e) => e.inputs(),
            },
            _ => {
                unreachable!()
            }
        };

        let input_ids = inputs
            .iter()
            .map(|i| match i {
                Input::Utxo(input) => *input.output_id(),
                _ => {
                    unreachable!()
                }
            })
            .collect();

        crate::node_api::core::get_outputs(self, input_ids).await
    }

    /// A generic send function for easily sending transaction or tagged data messages.
    pub fn message(&self) -> ClientMessageBuilder<'_> {
        ClientMessageBuilder::new(self)
    }

    /// Return a list of addresses from the signer regardless of their validity.
    pub fn get_addresses<'a>(&'a self, signer: &'a SignerHandle) -> GetAddressesBuilder<'a> {
        GetAddressesBuilder::new(signer).with_client(self)
    }

    /// Find all messages by provided message IDs.
    pub async fn find_messages(&self, message_ids: &[MessageId]) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // Use a `HashSet` to prevent duplicate message_ids.
        let mut message_ids_to_query = HashSet::<MessageId>::new();

        // Collect the `MessageId` in the HashSet.
        for message_id in message_ids {
            message_ids_to_query.insert(message_id.to_owned());
        }

        // Use `get_message_data()` API to get the `Message`.
        for message_id in message_ids_to_query {
            let message = self.get_message_data(&message_id).await?;
            messages.push(message);
        }
        Ok(messages)
    }

    /// Retries (promotes or reattaches) a message for provided message id. Message should only be
    /// retried only if they are valid and haven't been confirmed for a while.
    pub async fn retry(&self, message_id: &MessageId) -> Result<(MessageId, Message)> {
        // Get the metadata to check if it needs to promote or reattach
        let message_metadata = self.get_message_metadata(message_id).await?;
        if message_metadata.should_promote.unwrap_or(false) {
            self.promote_unchecked(message_id).await
        } else if message_metadata.should_reattach.unwrap_or(false) {
            self.reattach_unchecked(message_id).await
        } else {
            Err(Error::NoNeedPromoteOrReattach(message_id.to_string()))
        }
    }

    /// Retries (promotes or reattaches) a message for provided message id until it's included (referenced by a
    /// milestone). Default interval is 5 seconds and max attempts is 40. Returns the included message at first position
    /// and additional reattached messages
    pub async fn retry_until_included(
        &self,
        message_id: &MessageId,
        interval: Option<u64>,
        max_attempts: Option<u64>,
    ) -> Result<Vec<(MessageId, Message)>> {
        log::debug!("[retry_until_included]");
        // Attachments of the Message to check inclusion state
        let mut message_ids = vec![*message_id];
        // Reattached Messages that get returned
        let mut messages_with_id = Vec::new();
        for _ in 0..max_attempts.unwrap_or(40) {
            #[cfg(feature = "wasm")]
            {
                TimeoutFuture::new((interval.unwrap_or(5) * 1000).try_into().unwrap()).await;
            }
            #[cfg(not(feature = "wasm"))]
            sleep(Duration::from_secs(interval.unwrap_or(5))).await;
            // Check inclusion state for each attachment
            let message_ids_len = message_ids.len();
            let mut conflicting = false;
            for (index, msg_id) in message_ids.clone().iter().enumerate() {
                let message_metadata = self.get_message_metadata(msg_id).await?;
                if let Some(inclusion_state) = message_metadata.ledger_inclusion_state {
                    match inclusion_state {
                        LedgerInclusionStateDto::Included | LedgerInclusionStateDto::NoTransaction => {
                            // if original message, request it so we can return it on first position
                            if message_id == msg_id {
                                let mut included_and_reattached_messages =
                                    vec![(*message_id, self.get_message_data(message_id).await?)];
                                included_and_reattached_messages.extend(messages_with_id);
                                return Ok(included_and_reattached_messages);
                            } else {
                                // Move included message to first position
                                messages_with_id.rotate_left(index);
                                return Ok(messages_with_id);
                            }
                        }
                        // only set it as conflicting here and don't return, because another reattached message could
                        // have the included transaction
                        LedgerInclusionStateDto::Conflicting => conflicting = true,
                    };
                }
                // Only reattach or promote latest attachment of the message
                if index == message_ids_len - 1 {
                    if message_metadata.should_promote.unwrap_or(false) {
                        // Safe to unwrap since we iterate over it
                        self.promote_unchecked(message_ids.last().unwrap()).await?;
                    } else if message_metadata.should_reattach.unwrap_or(false) {
                        // Safe to unwrap since we iterate over it
                        let reattached = self.reattach_unchecked(message_ids.last().unwrap()).await?;
                        message_ids.push(reattached.0);
                        messages_with_id.push(reattached);
                    }
                }
            }
            // After we checked all our reattached messages, check if the transaction got reattached in another message
            // and confirmed
            if conflicting {
                let message = self.get_message_data(message_id).await?;
                if let Some(Payload::Transaction(transaction_payload)) = message.payload() {
                    let included_message = self.get_included_message(&transaction_payload.id()).await?;
                    let mut included_and_reattached_messages = vec![(included_message.id(), included_message)];
                    included_and_reattached_messages.extend(messages_with_id);
                    return Ok(included_and_reattached_messages);
                }
            }
        }
        Err(Error::TangleInclusionError(message_id.to_string()))
    }

    /// Function to consolidate all funds from a range of addresses to the address with the lowest index in that range
    /// Returns the address to which the funds got consolidated, if any were available
    pub async fn consolidate_funds(
        &self,
        signer: &SignerHandle,
        account_index: u32,
        address_range: Range<u32>,
    ) -> crate::Result<String> {
        crate::api::consolidate_funds(self, signer, account_index, address_range).await
    }

    /// Function to find inputs from addresses for a provided amount (useful for offline signing), ignoring outputs with
    /// additional unlock conditions
    pub async fn find_inputs(&self, addresses: Vec<String>, amount: u64) -> Result<Vec<UtxoInput>> {
        // Get outputs from node and select inputs
        let mut available_outputs = Vec::new();
        for address in addresses {
            available_outputs.extend_from_slice(
                &self
                    .get_address()
                    .outputs(vec![
                        QueryParameter::Address(address.to_string()),
                        QueryParameter::HasExpirationCondition(false),
                        QueryParameter::HasTimelockCondition(false),
                        QueryParameter::HasStorageDepositReturnCondition(false),
                    ])
                    .await?,
            );
        }

        let mut basic_outputs = Vec::new();

        for output_resp in available_outputs.into_iter() {
            let (amount, _) = ClientMessageBuilder::get_output_amount_and_address(&output_resp.output, None)?;
            basic_outputs.push((
                UtxoInput::new(
                    TransactionId::from_str(&output_resp.transaction_id)?,
                    output_resp.output_index,
                )?,
                amount,
            ));
        }
        basic_outputs.sort_by(|l, r| r.1.cmp(&l.1));

        let mut total_already_spent = 0;
        let mut selected_inputs = Vec::new();
        for (_offset, output_wrapper) in basic_outputs
            .into_iter()
            // Max inputs is 128
            .take(INPUT_COUNT_MAX.into())
            .enumerate()
        {
            // Break if we have enough funds and don't create dust for the remainder
            if total_already_spent == amount || total_already_spent >= amount {
                break;
            }
            selected_inputs.push(output_wrapper.0.clone());
            total_already_spent += output_wrapper.1;
        }

        if total_already_spent < amount {
            return Err(crate::Error::NotEnoughBalance(total_already_spent, amount));
        }

        Ok(selected_inputs)
    }

    /// Find all outputs based on the requests criteria. This method will try to query multiple nodes if
    /// the request amount exceeds individual node limit.
    pub async fn find_outputs(&self, outputs: &[UtxoInput], addresses: &[String]) -> Result<Vec<OutputResponse>> {
        let mut output_metadata =
            crate::node_api::core::get_outputs(self, outputs.iter().map(|output| *output.output_id()).collect())
                .await?;

        // Use `get_address()` API to get the address outputs first,
        // then collect the `UtxoInput` in the HashSet.
        for address in addresses {
            // Get output ids of outputs that can be controlled by this address without further unlock constraints
            let address_outputs = self
                .get_address()
                .outputs(vec![
                    QueryParameter::Address(address.to_string()),
                    QueryParameter::HasExpirationCondition(false),
                    QueryParameter::HasTimelockCondition(false),
                    QueryParameter::HasStorageDepositReturnCondition(false),
                ])
                .await?;
            output_metadata.extend(address_outputs.into_iter());
        }

        Ok(output_metadata.to_vec())
    }

    /// Reattaches messages for provided message id. Messages can be reattached only if they are valid and haven't been
    /// confirmed for a while.
    pub async fn reattach(&self, message_id: &MessageId) -> Result<(MessageId, Message)> {
        let metadata = self.get_message_metadata(message_id).await?;
        if metadata.should_reattach.unwrap_or(false) {
            self.reattach_unchecked(message_id).await
        } else {
            Err(Error::NoNeedPromoteOrReattach(message_id.to_string()))
        }
    }

    /// Reattach a message without checking if it should be reattached
    pub async fn reattach_unchecked(&self, message_id: &MessageId) -> Result<(MessageId, Message)> {
        // Get the Message object by the MessageID.
        let message = self.get_message_data(message_id).await?;
        let reattach_message = {
            #[cfg(feature = "wasm")]
            {
                let mut tips = self.get_tips().await?;
                tips.sort_unstable_by_key(|a| a.pack_to_vec());
                tips.dedup();
                let mut message_builder = MessageBuilder::<ClientMiner>::new(Parents::new(tips)?);
                if let Some(p) = message.payload().to_owned() {
                    message_builder = message_builder.with_payload(p.clone())
                }
                message_builder.finish().map_err(Error::MessageError)?
            }
            #[cfg(not(feature = "wasm"))]
            {
                finish_pow(self, message.payload().cloned()).await?
            }
        };

        // Post the modified
        let message_id = self.post_message(&reattach_message).await?;
        // Get message if we use remote PoW, because the node will change parents and nonce
        let msg = match self.get_local_pow().await {
            true => reattach_message,
            false => self.get_message_data(&message_id).await?,
        };
        Ok((message_id, msg))
    }

    /// Promotes a message. The method should validate if a promotion is necessary through get_message. If not, the
    /// method should error out and should not allow unnecessary promotions.
    pub async fn promote(&self, message_id: &MessageId) -> Result<(MessageId, Message)> {
        let metadata = self.get_message_metadata(message_id).await?;
        if metadata.should_promote.unwrap_or(false) {
            self.promote_unchecked(message_id).await
        } else {
            Err(Error::NoNeedPromoteOrReattach(message_id.to_string()))
        }
    }

    /// Promote a message without checking if it should be promoted
    pub async fn promote_unchecked(&self, message_id: &MessageId) -> Result<(MessageId, Message)> {
        // Create a new message (zero value message) for which one tip would be the actual message
        let mut tips = self.get_tips().await?;
        let min_pow_score = self.get_min_pow_score().await?;
        tips.push(*message_id);
        // Sort tips/parents
        tips.sort_unstable_by_key(|a| a.pack_to_vec());
        tips.dedup();

        let promote_message = MessageBuilder::<ClientMiner>::new(Parents::new(tips)?)
            .with_nonce_provider(self.get_pow_provider().await, min_pow_score)
            .finish()
            .map_err(|_| Error::TransactionError)?;

        let message_id = self.post_message(&promote_message).await?;
        // Get message if we use remote PoW, because the node will change parents and nonce
        let msg = match self.get_local_pow().await {
            true => promote_message,
            false => self.get_message_data(&message_id).await?,
        };
        Ok((message_id, msg))
    }

    /// Returns checked local time and milestone index.
    pub async fn get_time_and_milestone_checked(&self) -> Result<(u64, u32)> {
        let local_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let status_response = self.get_info().await?.nodeinfo.status;
        let latest_ms_timestamp = status_response.latest_milestone_timestamp;
        // Check the local time is in the range of +-5 minutes of the node to prevent locking funds by accident
        if !(latest_ms_timestamp - FIVE_MINUTES_IN_SECONDS..latest_ms_timestamp + FIVE_MINUTES_IN_SECONDS)
            .contains(&local_time)
        {
            return Err(Error::TimeNotSynced(local_time, latest_ms_timestamp));
        }
        Ok((local_time, status_response.latest_milestone_index))
    }

    //////////////////////////////////////////////////////////////////////
    // Utils
    //////////////////////////////////////////////////////////////////////

    /// Transforms bech32 to hex
    pub fn bech32_to_hex(bech32: &str) -> crate::Result<String> {
        bech32_to_hex(bech32)
    }

    /// Transforms a hex encoded address to a bech32 encoded address
    pub async fn hex_to_bech32(&self, hex: &str, bech32_hrp: Option<&str>) -> crate::Result<String> {
        let bech32_hrp = match bech32_hrp {
            Some(hrp) => hrp.into(),
            None => self.get_bech32_hrp().await?,
        };
        hex_to_bech32(hex, &bech32_hrp)
    }

    /// Transforms a hex encoded public key to a bech32 encoded address
    pub async fn hex_public_key_to_bech32_address(&self, hex: &str, bech32_hrp: Option<&str>) -> crate::Result<String> {
        let bech32_hrp = match bech32_hrp {
            Some(hrp) => hrp.into(),
            None => self.get_bech32_hrp().await?,
        };
        hex_public_key_to_bech32_address(hex, &bech32_hrp)
    }

    /// Returns a valid Address parsed from a String.
    pub fn parse_bech32_address(address: &str) -> crate::Result<Address> {
        parse_bech32_address(address)
    }

    /// Checks if a String is a valid bech32 encoded address.
    pub fn is_address_valid(address: &str) -> bool {
        is_address_valid(address)
    }

    /// Generates a new mnemonic.
    pub fn generate_mnemonic() -> Result<String> {
        generate_mnemonic()
    }

    /// Returns a seed for a mnemonic.
    pub fn mnemonic_to_seed(mnemonic: &str) -> Result<Seed> {
        mnemonic_to_seed(mnemonic)
    }

    /// Returns a hex encoded seed for a mnemonic.
    pub fn mnemonic_to_hex_seed(mnemonic: &str) -> Result<String> {
        mnemonic_to_hex_seed(mnemonic)
    }

    /// UTF-8 encodes the `tag` of a given TaggedDataPayload.
    pub fn tag_to_utf8(payload: &TaggedDataPayload) -> Result<String> {
        Ok(String::from_utf8(payload.tag().to_vec())
            .map_err(|_| Error::TaggedDataError("found invalid UTF-8".to_string()))?)
    }

    /// UTF-8 encodes the `data` of a given TaggedDataPayload.
    pub fn data_to_utf8(payload: &TaggedDataPayload) -> Result<String> {
        Ok(String::from_utf8(payload.data().to_vec())
            .map_err(|_| Error::TaggedDataError("found invalid UTF-8".to_string()))?)
    }

    /// UTF-8 encodes both the `tag` and `data` of a given TaggedDataPayload.
    pub fn tagged_data_to_utf8(payload: &TaggedDataPayload) -> Result<(String, String)> {
        Ok((Client::tag_to_utf8(&payload)?, Client::data_to_utf8(&payload)?))
    }
}
