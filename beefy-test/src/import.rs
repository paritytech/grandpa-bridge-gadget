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

use sc_client_api::backend::TransactionFor;
use sp_blockchain::well_known_cache_keys;
use sp_consensus::{
	block_import::JustificationImport,
	import_queue::{CacheKeyId, Verifier},
	BlockCheckParams, BlockImport, BlockImportParams, BlockOrigin, ForkChoiceStrategy, ImportResult,
};
use sp_core::H256;
use sp_runtime::{
	generic::{BlockId, OpaqueDigestItemId},
	traits::{Block, Header, NumberFor},
	Justification, Justifications,
};

use substrate_test_runtime_client::{runtime, Backend};

use futures::lock::Mutex as AsyncMutex;
use parking_lot::Mutex;

use crate::Client;

pub trait AnyTransaction:
	BlockImport<runtime::Block, Transaction = TransactionFor<Backend, runtime::Block>, Error = sp_consensus::Error>
	+ Send
	+ Sync
	+ Clone
{
	// empty
}

impl<T> AnyTransaction for T
where
	T: BlockImport<runtime::Block, Transaction = TransactionFor<Backend, runtime::Block>, Error = sp_consensus::Error>
		+ Send
		+ Sync
		+ Clone,
{
	// empty
}

/// Implements [`sp_consensus::block_import::BlockImport`] for the any transaction type.
#[derive(Clone)]
pub struct AnyBlockImport<BI> {
	inner: BI,
}

impl<I> AnyBlockImport<I> {
	pub fn new(inner: I) -> Self {
		Self { inner }
	}
}

#[async_trait::async_trait]
impl<BI> BlockImport<runtime::Block> for AnyBlockImport<BI>
where
	BI: BlockImport<runtime::Block, Error = sp_consensus::Error> + Send + Sync,
	BI::Transaction: Send,
{
	type Error = sp_consensus::Error;
	type Transaction = ();

	/// Check block preconditions
	async fn check_block(&mut self, block: BlockCheckParams<runtime::Block>) -> Result<ImportResult, Self::Error> {
		self.inner.check_block(block).await
	}

	/// Import a block
	async fn import_block(
		&mut self,
		block: BlockImportParams<runtime::Block, Self::Transaction>,
		cache: HashMap<CacheKeyId, Vec<u8>>,
	) -> Result<ImportResult, Self::Error> {
		self.inner
			.import_block(block.clear_storage_changes_and_mutate(), cache)
			.await
	}
}

/// A Verifier that accepts all justifications and passes them on for import.
///
/// Block finality and fork choice strategy are configurable.
#[derive(Clone)]
pub struct PassThroughVerifier {
	finalized: bool,
	fork_choice: ForkChoiceStrategy,
}

impl PassThroughVerifier {
	pub fn new(finalized: bool) -> Self {
		Self {
			finalized,
			fork_choice: ForkChoiceStrategy::LongestChain,
		}
	}
}

#[async_trait::async_trait]
impl<B> Verifier<B> for PassThroughVerifier
where
	B: Block,
{
	async fn verify(
		&mut self,
		origin: BlockOrigin,
		header: B::Header,
		justifications: Option<Justifications>,
		body: Option<Vec<B::Extrinsic>>,
	) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
		let maybe_keys = header
			.digest()
			.log(|l| l.try_as_raw(OpaqueDigestItemId::Consensus(b"smpl")))
			.map(|l| vec![(well_known_cache_keys::AUTHORITIES, l.to_vec())]);

		let mut import = BlockImportParams::new(origin, header);
		import.body = body;
		import.finalized = self.finalized;
		import.justifications = justifications;
		import.fork_choice = Some(self.fork_choice);

		Ok((import, maybe_keys))
	}
}

/// A [`sp_consensus::block_import::JustificationImport`] implementation that
/// will always finalize the imported block.
pub struct Finalizer(pub Client);

#[async_trait::async_trait]
impl JustificationImport<runtime::Block> for Finalizer {
	type Error = sp_consensus::Error;

	async fn on_start(&mut self) -> Vec<(H256, NumberFor<runtime::Block>)> {
		Vec::new()
	}

	async fn import_justification(
		&mut self,
		hash: H256,
		_number: NumberFor<runtime::Block>,
		justification: Justification,
	) -> Result<(), Self::Error> {
		self.0
			.finalize_block(BlockId::Hash(hash), Some(justification), true)
			.map_err(|_| sp_consensus::Error::InvalidJustification)
	}
}

/// Verifier implementation for tracking failed verifications
pub struct TrackingVerifier<B>
where
	B: Block,
{
	inner: Arc<AsyncMutex<Box<dyn Verifier<B>>>>,
	failed: Arc<Mutex<HashMap<B::Hash, String>>>,
}

impl<B> TrackingVerifier<B>
where
	B: Block,
{
	pub fn new(verifier: impl Verifier<B> + 'static) -> Self {
		TrackingVerifier {
			inner: Arc::new(AsyncMutex::new(Box::new(verifier))),
			failed: Default::default(),
		}
	}
}

#[async_trait::async_trait]
impl<B> Verifier<B> for TrackingVerifier<B>
where
	B: Block,
{
	async fn verify(
		&mut self,
		origin: BlockOrigin,
		header: B::Header,
		justifications: Option<Justifications>,
		body: Option<Vec<B::Extrinsic>>,
	) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
		let hash = header.hash();

		self.inner
			.lock()
			.await
			.verify(origin, header, justifications, body)
			.await
			.map_err(|e| {
				self.failed.lock().insert(hash, e.clone());
				e
			})
	}
}

impl<B> Clone for TrackingVerifier<B>
where
	B: Block,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
			failed: self.failed.clone(),
		}
	}
}
