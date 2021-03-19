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

use std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	sync::Arc,
};

use beefy_primitives::{
	BeefyApi, Commitment, ConsensusLog, MmrRootHash, SignedCommitment, ValidatorSet, ValidatorSetId, BEEFY_ENGINE_ID,
	KEY_TYPE,
};
use codec::{Codec, Decode, Encode};
use futures::{future, FutureExt, Stream, StreamExt};
use hex::ToHex;
use log::{debug, error, info, trace, warn};
use parking_lot::Mutex;

use sc_client_api::{Backend, FinalityNotification, FinalityNotifications};
use sc_network_gossip::GossipEngine;

use sp_api::BlockId;
use sp_application_crypto::{AppPublic, Public};
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};
use sp_runtime::{
	generic::OpaqueDigestItemId,
	traits::{Block, Hash, Header, NumberFor, Zero},
};

use crate::{error, notification, round, Client};

/// Gossip engine messages topic
pub(crate) fn topic<B: Block>() -> B::Hash
where
	B: Block,
{
	<<B::Header as Header>::Hashing as Hash>::hash(b"beefy")
}

#[derive(Debug, Decode, Encode)]
struct VoteMessage<Hash, Number, Id, Signature> {
	commitment: Commitment<Number, Hash>,
	id: Id,
	signature: Signature,
}
/// Worker lifecycle state
enum State {
	/// A new worker that still needs to be initialized.
	New,
	/// A worker that has been initialized
	Initialized,
}

pub(crate) struct BeefyWorker<B, Id, S, C, BE, P>
where
	B: Block,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
	C: Client<B, BE, P>,
{
	state: State,
	local_id: Option<Id>,
	key_store: SyncCryptoStorePtr,
	min_interval: u32,
	rounds: round::Rounds<MmrRootHash, NumberFor<B>, Id, S>,
	finality_notifications: FinalityNotifications<B>,
	gossip_engine: Arc<Mutex<GossipEngine<B>>>,
	signed_commitment_sender: notification::BeefySignedCommitmentSender<B, S>,
	best_finalized_block: NumberFor<B>,
	best_block_voted_on: NumberFor<B>,
	validator_set_id: ValidatorSetId,
	client: Arc<C>,
	_backend: PhantomData<BE>,
	_pair: PhantomData<P>,
}

impl<B, Id, S, C, BE, P> BeefyWorker<B, Id, S, C, BE, P>
where
	B: Block,
	Id: Public + Debug,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
	C: Client<B, BE, P>,
	C::Api: BeefyApi<B, P::Public>,
{
	pub(crate) fn new(
		key_store: SyncCryptoStorePtr,
		gossip_engine: GossipEngine<B>,
		signed_commitment_sender: notification::BeefySignedCommitmentSender<B, S>,
		client: Arc<C>,
	) -> Self {
		let at = BlockId::hash(client.info().best_hash);

		let validator_set = client
			.runtime_api()
			.validator_set(&at)
			.expect("Failed to get BEEFY validator set");

		let _local_id = match validator_set
			.validators
			.iter()
			.find(|id| SyncCryptoStore::has_keys(&*key_store, &[(id.to_raw_vec(), KEY_TYPE)]))
		{
			Some(id) => {
				info!(target: "beefy", "🥩 Starting BEEFY worker with local id: {:?}", id);
				Some(id.clone())
			}
			None => {
				info!(target: "beefy", "🥩 No local id found, BEEFY worker will be gossip only.");
				None
			}
		};

		BeefyWorker {
			state: State::New,
			local_id: None,
			key_store,
			min_interval: 2,
			rounds: round::Rounds::new(vec![]),
			finality_notifications: client.finality_notification_stream(),
			gossip_engine: Arc::new(Mutex::new(gossip_engine)),
			signed_commitment_sender,
			best_finalized_block: client.info().finalized_number,
			best_block_voted_on: Zero::zero(),
			validator_set_id: validator_set.id,
			client,
			_backend: PhantomData,
			_pair: PhantomData,
		}
	}
}

impl<B, Id, S, C, BE, P> BeefyWorker<B, Id, S, C, BE, P>
where
	B: Block,
	Id: Codec + Debug + PartialEq + Public,
	S: Clone + Codec + Debug + PartialEq + std::convert::TryFrom<Vec<u8>>,
	BE: Backend<B>,
	P: sp_core::Pair,
	P::Public: AppPublic + Codec,
	P::Signature: Clone + Codec + Debug + PartialEq + TryFrom<Vec<u8>>,
	C: Client<B, BE, P>,
	C::Api: BeefyApi<B, P::Public>,
{
	fn should_vote_on(&self, number: NumberFor<B>) -> bool {
		use sp_runtime::{traits::Saturating, SaturatedConversion};

		// we only vote as a validator
		if self.local_id.is_none() {
			return false;
		}

		let diff = self.best_finalized_block.saturating_sub(self.best_block_voted_on);
		let diff = diff.saturated_into::<u32>();
		let next_power_of_two = (diff / 2).next_power_of_two();
		let next_block_to_vote_on = self.best_block_voted_on + self.min_interval.max(next_power_of_two).into();

		trace!(
			target: "beefy",
			"should_vote_on: #{:?}, diff: {:?}, next_power_of_two: {:?}, next_block_to_vote_on: #{:?}",
			number,
			diff,
			next_power_of_two,
			next_block_to_vote_on,
		);

		number == next_block_to_vote_on
	}

	fn sign_commitment(&self, id: &Id, commitment: &[u8]) -> Result<S, error::Error<Id>> {
		let sig = SyncCryptoStore::sign_with(&*self.key_store, KEY_TYPE, &id.to_public_crypto_pair(), &commitment)
			.map_err(|e| error::Error::CannotSign((*id).clone(), e.to_string()))?
			.ok_or_else(|| error::Error::CannotSign((*id).clone(), "No key in KeyStore found".into()))?;

		let sig = sig
			.clone()
			.try_into()
			.map_err(|_| error::Error::InvalidSignature(sig.encode_hex(), (*id).clone()))?;

		Ok(sig)
	}

	fn handle_finality_notification(&mut self, notification: FinalityNotification<B>) {
		debug!(target: "beefy", "🥩 Finality notification: {:?}", notification);

		if self.should_vote_on(*notification.header.number()) {
			let local_id = if let Some(id) = &self.local_id {
				id
			} else {
				warn!(target: "beefy", "🥩 Missing validator id - can't vote for: {:?}", notification.header.hash());
				return;
			};

			let mmr_root = if let Some(hash) = find_mmr_root_digest::<B, Id>(&notification.header) {
				hash
			} else {
				warn!(target: "beefy", "🥩 No MMR root digest found for: {:?}", notification.header.hash());
				return;
			};

			if let Some(new) = find_authorities_change::<B, Id>(&notification.header) {
				debug!(target: "beefy", "🥩 New validator set: {:?}", new);
				self.validator_set_id = new.id;
			};

			let commitment = Commitment {
				payload: mmr_root,
				block_number: notification.header.number(),
				validator_set_id: self.validator_set_id,
			};

			let signature = match self.sign_commitment(local_id, commitment.encode().as_ref()) {
				Ok(sig) => sig,
				Err(err) => {
					warn!(target: "beefy", "🥩 Error signing commitment: {:?}", err);
					return;
				}
			};

			self.best_block_voted_on = *notification.header.number();

			let message = VoteMessage {
				commitment,
				id: local_id.clone(),
				signature,
			};

			self.gossip_engine
				.lock()
				.gossip_message(topic::<B>(), message.encode(), false);

			debug!(target: "beefy", "🥩 Sent vote message: {:?}", message);

			self.handle_vote(
				(message.commitment.payload, *message.commitment.block_number),
				(message.id, message.signature),
			);
		}

		self.best_finalized_block = *notification.header.number();
	}

	fn handle_vote(&mut self, round: (MmrRootHash, NumberFor<B>), vote: (Id, S)) {
		// TODO: validate signature
		let vote_added = self.rounds.add_vote(round, vote);

		if vote_added && self.rounds.is_done(&round) {
			if let Some(signatures) = self.rounds.drop(&round) {
				let commitment = Commitment {
					payload: round.0,
					block_number: round.1,
					validator_set_id: self.validator_set_id,
				};

				let signed_commitment = SignedCommitment { commitment, signatures };

				info!(target: "beefy", "🥩 Round #{} concluded, committed: {:?}.", round.1, signed_commitment);

				self.signed_commitment_sender.notify(signed_commitment);
			}
		}
	}

	pub(crate) async fn run(mut self) {
		let mut votes = Box::pin(self.gossip_engine.lock().messages_for(topic::<B>()).filter_map(
			|notification| async move {
				debug!(target: "beefy", "🥩 Got vote message: {:?}", notification);

				VoteMessage::<MmrRootHash, NumberFor<B>, Id, S>::decode(&mut &notification.message[..]).ok()
			},
		));

		loop {
			let engine = self.gossip_engine.clone();
			let gossip_engine = future::poll_fn(|cx| engine.lock().poll_unpin(cx));

			futures::select! {
				notification = self.finality_notifications.next().fuse() => {
					if let Some(notification) = notification {
						self.handle_finality_notification(notification);
					} else {
						return;
					}
				},
				vote = votes.next() => {
					if let Some(vote) = vote {
						self.handle_vote(
							(vote.commitment.payload, vote.commitment.block_number),
							(vote.id, vote.signature),
						);
					} else {
						return;
					}
				},
				_ = gossip_engine.fuse() => {
					error!(target: "beefy", "🥩 Gossip engine has terminated.");
					return;
				}
			}
		}
	}
}

/// Extract the MMR root hash from a digest in the given header, if it exists.
fn find_mmr_root_digest<B, Id>(header: &B::Header) -> Option<MmrRootHash>
where
	B: Block,
	Id: Codec,
{
	header.digest().logs().iter().find_map(|log| {
		match log.try_to::<ConsensusLog<Id>>(OpaqueDigestItemId::Consensus(&BEEFY_ENGINE_ID)) {
			Some(ConsensusLog::MmrRoot(root)) => Some(root),
			_ => None,
		}
	})
}

/// Scan the `header` digest log for a BEEFY validator set change. Return either the new
/// validator set or `None` in case no validator set change has been signaled.
fn find_authorities_change<B, Id>(header: &B::Header) -> Option<ValidatorSet<Id>>
where
	B: Block,
	Id: Codec,
{
	let id = OpaqueDigestItemId::Consensus(&BEEFY_ENGINE_ID);

	let filter = |log: ConsensusLog<Id>| match log {
		ConsensusLog::AuthoritiesChange(validator_set) => Some(validator_set),
		_ => None,
	};

	header.digest().convert_first(|l| l.try_to(id).and_then(filter))
}
