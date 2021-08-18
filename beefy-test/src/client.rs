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

use std::{collections::HashMap, sync::Arc};

use sc_client_api::backend::Finalizer;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult, LongestChain};
use sp_blockchain::Info;
use sp_consensus::CacheKeyId;
use sp_runtime::{generic::BlockId, Justification};

use substrate_test_runtime::Block;
use substrate_test_runtime_client::{Backend, TestClient, TestClientBuilder, TestClientBuilderExt};

use crate::import::AnyBlockImport;

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;

/// A test client.
#[derive(Clone)]
pub struct Client {
	pub(crate) inner: Arc<TestClient>,
	pub(crate) backend: Arc<Backend>,
	pub(crate) chain: LongestChain<substrate_test_runtime_client::Backend, Block>,
}

impl Client {
	/// Return a new test client.
	pub fn new() -> Client {
		let backend = Arc::new(Backend::new_test(std::u32::MAX, std::u64::MAX));
		let builder = TestClientBuilder::with_backend(backend);
		let backend = builder.backend();

		let (client, chain) = builder.build_with_longest_chain();

		Client {
			inner: Arc::new(client),
			backend,
			chain,
		}
	}
}

impl Default for Client {
	fn default() -> Self {
		Self::new()
	}
}

impl Client {
	/// Implementation for [`sc_client_api::backend::Finalizer`]
	pub fn finalize_block(
		&self,
		id: BlockId<Block>,
		justification: Option<Justification>,
		notify: bool,
	) -> sp_blockchain::Result<()> {
		self.inner.finalize_block(id, justification, notify)
	}

	/// Return a clone of the client as [`crate::AnyBlockImport`]
	pub fn as_block_import(&self) -> AnyBlockImport<Self> {
		AnyBlockImport::new(self.clone())
	}

	/// Return a clone of the inner test client
	pub fn as_inner(&self) -> Arc<TestClient> {
		self.inner.clone()
	}

	/// Return a clone of the client [`substrate_test_runtime_client::Backend`]
	pub fn as_backend(&self) -> Arc<Backend> {
		self.backend.clone()
	}

	/// Return client blockchain info
	pub fn info(&self) -> Info<Block> {
		self.inner.chain_info()
	}

	/// Return a clone of the client longest chain
	pub fn chain(&self) -> LongestChain<substrate_test_runtime_client::Backend, Block> {
		self.chain.clone()
	}
}

#[async_trait::async_trait]
impl BlockImport<Block> for Client {
	type Error = sp_consensus::Error;

	type Transaction = ();

	/// Check block preconditions
	async fn check_block(&mut self, block: BlockCheckParams<Block>) -> Result<ImportResult, Self::Error> {
		self.inner.check_block(block).await
	}

	/// Import a block
	async fn import_block(
		&mut self,
		block: BlockImportParams<Block, Self::Transaction>,
		cache: HashMap<CacheKeyId, Vec<u8>>,
	) -> Result<ImportResult, Self::Error> {
		self.inner
			.import_block(block.clear_storage_changes_and_mutate(), cache)
			.await
	}
}
