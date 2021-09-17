// Copyright (C) 2020 Parity Technologies (UK) Ltd.
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

//! RPC API for BEEFY.

#![warn(missing_docs)]

use beefy_gadget::notification::BeefySignedCommitmentStream;
use futures::{FutureExt, SinkExt, StreamExt};
use jsonrpsee::{proc_macros::rpc, types::RpcResult, SubscriptionSink};
use log::warn;
use sp_runtime::traits::Block as BlockT;
use sc_rpc::SubscriptionTaskExecutor;

mod notification;

/// Provides RPC methods for interacting with BEEFY.
#[rpc(client, server, namespace = "beefy")]
pub trait BeefyApi<Notification, Hash> {
	/// Returns the block most recently finalized by BEEFY, alongside side its justification.
	#[subscription(
		name = "subscribeJustifications"
		aliases = "beefy_justifications",
		item = Notification,
	)]
	fn subscribe_justifications(&self) -> RpcResult<()>;
}

/// Implements the BeefyApi RPC trait for interacting with BEEFY.
pub struct BeefyRpcHandler<Block: BlockT> {
	signed_commitment_stream: BeefySignedCommitmentStream<Block>,
	executor: SubscriptionTaskExecutor,
}

impl<Block> BeefyRpcHandler<Block>
where
	Block: BlockT,
{
	/// Creates a new BeefyRpcHandler instance.
	pub fn new(
		signed_commitment_stream: BeefySignedCommitmentStream<Block>,
		executor: SubscriptionTaskExecutor
	) -> Self {
		Self {
			signed_commitment_stream,
			executor,
		}
	}
}

impl<Block> BeefyApiServer<notification::SignedCommitment, Block> for BeefyRpcHandler<Block>
where
	Block: BlockT,
{

	fn subscribe_justifications(
		&self,
		mut sink: SubscriptionSink,
	) -> RpcResult<()> {
		// let stream = self
		//     .signed_commitment_stream
		//     .subscribe()
		//     .map(|x| Ok::<_, ()>(Ok(notification::SignedCommitment::new::<Block>(x))));

		/*self.executor.spawn(
			stream
				.for_each(
				.forward(sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e)))
				.map(|_| ())
		);*/
		Ok(())
	}
}
