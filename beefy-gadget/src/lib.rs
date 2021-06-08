// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{convert::TryFrom, fmt::Debug, sync::Arc};

use codec::Codec;
use keystore::BeefyKeystore;
use log::debug;
use prometheus::Registry;

use sc_client_api::{Backend, BlockchainEvents, Finalizer};
use sc_network_gossip::{GossipEngine, Network as GossipNetwork};

use sp_api::ProvideRuntimeApi;
use sp_application_crypto::AppPublic;
use sp_blockchain::HeaderBackend;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::Block;

use beefy_primitives::{ecdsa, BeefyApi};

mod gossip;
mod keystore;
mod metrics;
mod round;
mod worker;

pub mod error;
pub mod notification;

pub const BEEFY_PROTOCOL_NAME: &str = "/paritytech/beefy/1";

/// Returns the configuration value to put in
/// [`sc_network::config::NetworkConfiguration::extra_sets`].
pub fn beefy_peers_set_config() -> sc_network::config::NonDefaultSetConfig {
	let mut cfg = sc_network::config::NonDefaultSetConfig::new(BEEFY_PROTOCOL_NAME.into(), 1024 * 1024);
	cfg.allow_non_reserved(25, 25);
	cfg
}

/// A convenience BEEFY client trait that defines all the type bounds a BEEFY client
/// has to satisfy. Ideally that should actually be a trait alias. Unfortunately as
/// of today, Rust does not allow a type alias to be used as a trait bound. Tracking
/// issue is <https://github.com/rust-lang/rust/issues/41517>.
pub trait Client<B, BE, P>:
	BlockchainEvents<B> + HeaderBackend<B> + Finalizer<B, BE> + ProvideRuntimeApi<B> + Send + Sync
where
	B: Block,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
{
	// empty
}

impl<B, BE, P, T> Client<B, BE, P> for T
where
	B: Block,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
	T: BlockchainEvents<B> + HeaderBackend<B> + Finalizer<B, BE> + ProvideRuntimeApi<B> + Send + Sync,
{
	// empty
}

/// BEEFY gadget initialization parameters.
pub struct BeefyParams<B, P, BE, C, N>
where
	B: Block,
	P: sp_core::Pair,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
{
	/// BEEFY client
	pub client: Arc<C>,
	/// Client Backend
	pub backend: Arc<BE>,
	/// Local key store
	pub key_store: Option<Arc<dyn BeefyKeystore<P>>>,
	/// Gossip network
	pub network: N,
	/// BEEFY signed commitment sender
	pub signed_commitment_sender: notification::BeefySignedCommitmentSender<B, P::Signature>,
	/// Minimal delta between blocks, BEEFY should vote for
	pub min_block_delta: u32,
	/// Prometheus metric registry
	pub prometheus_registry: Option<Registry>,
}

/// Start the BEEFY gadget.
///
/// This is a thin shim around running and awaiting a BEEFY worker.
pub async fn start_beefy_gadget<B, P, BE, C, N>(beefy_params: BeefyParams<B, P, BE, C, N>)
where
	B: Block,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
	C: Client<B, BE, P>,
	C::Api: BeefyApi<B, P::Public>,
	N: GossipNetwork<B> + Clone + Send + 'static,
{
	let BeefyParams {
		client,
		backend,
		key_store,
		network,
		signed_commitment_sender,
		min_block_delta,
		prometheus_registry,
	} = beefy_params;

	let gossip_validator = Arc::new(gossip::BeefyGossipValidator::new());
	let gossip_engine = GossipEngine::new(network, BEEFY_PROTOCOL_NAME, gossip_validator.clone(), None);

	let metrics = prometheus_registry
		.as_ref()
		.map(metrics::Metrics::register)
		.and_then(|result| match result {
			Ok(metrics) => {
				debug!(target: "beefy", "🥩 Registered metrics");
				Some(metrics)
			}
			Err(err) => {
				debug!(target: "beefy", "🥩 Failed to register metrics: {:?}", err);
				None
			}
		});

	let worker_params = worker::WorkerParams {
		client,
		backend,
		key_store,
		signed_commitment_sender,
		gossip_engine,
		gossip_validator,
		min_block_delta,
		metrics,
	};

	let worker = worker::BeefyWorker::<_, _, _, _>::new(worker_params);

	worker.run().await
}
