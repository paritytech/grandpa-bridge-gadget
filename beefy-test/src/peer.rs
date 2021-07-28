// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

#![allow(dead_code)]

use std::{borrow::Cow, pin::Pin};

use sc_block_builder::BlockBuilderProvider;
use sc_client_api::{FinalityNotification, HeaderBackend};
use sc_consensus::LongestChain;
use sc_network::{Multiaddr, NetworkWorker};
use sp_consensus::BlockOrigin;
use sp_core::H256;
use sp_runtime::{generic::BlockId, traits::Header};

use substrate_test_runtime::{Block, Hash};
use substrate_test_runtime_client::{Backend, ClientBlockImportExt};

use crate::Client;

use futures::{
	executor::{self},
	Stream,
};
use libp2p::PeerId;
use log::trace;

type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

#[derive(Default, Clone)]
/// Configuration for a network peer
pub struct PeerConfig {
	/// Set of notification protocols a peer should participate in.
	pub protocols: Vec<Cow<'static, str>>,
	/// Is peer an authority or a regualr node
	pub is_authority: bool,
}

/// A network peer
pub struct Peer {
	pub(crate) client: Client,
	pub(crate) select_chain: Option<LongestChain<Backend, Block>>,
	pub(crate) network: NetworkWorker<Block, Hash>,
	pub(crate) finality_notification_stream: BoxStream<FinalityNotification<Block>>,
	pub(crate) listen_addr: Multiaddr,
}

impl Peer {
	/// Return unique peer id
	pub fn id(&self) -> PeerId {
		*self.network.service().local_peer_id()
	}

	/// Return a reference to the network, i.e. the peer's network worker
	pub fn network(&self) -> &NetworkWorker<Block, Hash> {
		&self.network
	}

	/// Return a reference to the peer's client
	pub fn client(&self) -> &Client {
		&self.client
	}

	/// Return the number of peers this peer is connected to
	pub fn connected_peers(&self) -> usize {
		self.network.num_connected_peers()
	}

	/// Return whether peer is currently syncing
	pub fn is_syncing(&self) -> bool {
		self.network.service().is_major_syncing()
	}

	/// Add a new block at best block.
	///
	/// Adding a new block will push the block through the block import pipeline.
	pub fn add_block(&mut self) -> Hash {
		let best = self.client.inner.info().best_hash;

		self.blocks_at(BlockId::Hash(best), 1)
	}

	/// Add `count` blocks at best block
	///
	/// Adding blocks will push them through the block import pipeline.
	pub fn add_blocks(&mut self, count: usize) -> Hash {
		let best = self.client.inner.info().best_hash;

		self.blocks_at(BlockId::Hash(best), count)
	}

	fn blocks_at(&mut self, at: BlockId<Block>, count: usize) -> H256 {
		let mut client = self.client.inner.clone();

		let mut best: H256 = [0u8; 32].into();

		for _ in 0..count {
			let block = client
				.new_block(Default::default())
				.expect("failed create a new block")
				.build()
				.expect("failed to build block")
				.block;

			let hash = block.header.hash();

			trace!(target: "beefy-test", "Block {} #{} parent: {}", hash, block.header.number, at);

			executor::block_on(client.import(BlockOrigin::File, block)).expect("block import failed");

			self.network.service().announce_block(hash, None);

			best = hash;
		}

		self.network.new_best_block_imported(
			best,
			*client.header(&BlockId::Hash(best)).ok().flatten().unwrap().number(),
		);

		best
	}
}
