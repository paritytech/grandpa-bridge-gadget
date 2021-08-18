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

use super::Client;

use sp_consensus::BlockOrigin;
use sp_runtime::{ConsensusEngineId, Digest, DigestItem, Justification, Justifications};

use sc_block_builder::BlockBuilderProvider;
use sc_client_api::{BlockchainEvents, HeaderBackend};

use substrate_test_runtime_client::prelude::*;

use futures::executor;

const ENGINE_ID: ConsensusEngineId = *b"BEEF";

#[tokio::test]
async fn import() {
	sp_tracing::try_init_simple();

	let mut client = Client::new();

	let block = client
		.inner
		.new_block(Default::default())
		.unwrap()
		.build()
		.unwrap()
		.block;

	let _ = client.inner.import(BlockOrigin::File, block).await;

	let info = client.inner.info();

	assert_eq!(1, info.best_number);
	assert_eq!(0, info.finalized_number);
}

#[tokio::test]
async fn import_blocks() {
	sp_tracing::try_init_simple();

	let mut client = Client::new();

	for _ in 0..10 {
		let block = client
			.inner
			.new_block(Default::default())
			.unwrap()
			.build()
			.unwrap()
			.block;

		let _ = client.inner.import(BlockOrigin::File, block).await;
	}

	let info = client.inner.info();

	assert_eq!(10, info.best_number);
	assert_eq!(0, info.finalized_number);
}

#[tokio::test]
async fn import_finalized() {
	sp_tracing::try_init_simple();

	let mut client = Client::new();

	let block = client
		.inner
		.new_block(Default::default())
		.unwrap()
		.build()
		.unwrap()
		.block;

	let _ = client.inner.import_as_final(BlockOrigin::File, block).await;

	let info = client.inner.info();

	assert_eq!(1, info.best_number);
	assert_eq!(1, info.finalized_number);
}

#[tokio::test]
async fn import_justification() {
	sp_tracing::try_init_simple();

	let mut client = Client::new();

	let block = client
		.inner
		.new_block(Default::default())
		.unwrap()
		.build()
		.unwrap()
		.block;

	let j: Justification = (ENGINE_ID, vec![1, 2, 3]);

	let j = Justifications::from(j);

	let _ = client.inner.import_justified(BlockOrigin::File, block, j).await;

	let info = client.inner.info();

	assert_eq!(1, info.best_number);
	assert_eq!(1, info.finalized_number);
}

#[tokio::test]
async fn finality_notification() {
	sp_tracing::try_init_simple();

	let mut client = Client::new();

	let mut finality_stream = executor::block_on_stream(client.inner.finality_notification_stream());

	let digest = Digest {
		logs: vec![DigestItem::Consensus(ENGINE_ID, vec![1, 2, 3])],
	};

	let block = client.inner.new_block(digest).unwrap().build().unwrap().block;

	let _ = client.inner.import_as_final(BlockOrigin::NetworkBroadcast, block).await;

	let import_hash = client.info().best_hash;
	let finality_notification = finality_stream.next().unwrap();
	assert_eq!(import_hash, finality_notification.hash);

	let item = DigestItem::Consensus(ENGINE_ID, vec![1, 2, 3]);
	assert_eq!(item, finality_notification.header.digest.logs[0]);
}
