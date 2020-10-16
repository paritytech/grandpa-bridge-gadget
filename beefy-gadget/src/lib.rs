use std::collections::BTreeMap;
use std::sync::Arc;

use futures::{FutureExt, Stream, StreamExt};
use log::info;
use parity_scale_codec::{Codec, Decode, Encode};

use sc_client_api::{Backend as BackendT, BlockchainEvents, FinalityNotification, Finalizer};
use sc_network_gossip::{
	GossipEngine, Network as GossipNetwork, ValidationResult as GossipValidationResult,
	Validator as GossipValidator, ValidatorContext as GossipValidatorContext,
};
use sp_consensus::SyncOracle as SyncOracleT;
use sp_runtime::{
	traits::{Block as BlockT, Hash as HashT, Header as HeaderT},
	ConsensusEngineId, KeyTypeId,
};

pub const BEEFY_ENGINE_ID: ConsensusEngineId = *b"BEEF";
pub const BEEFY_PROTOCOL_NAME: &'static str = "/paritytech/beefy/1";

/// Key type for BEEFY module.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"beef");

mod app {
	use sp_application_crypto::{app_crypto, ecdsa};
	app_crypto!(ecdsa, super::KEY_TYPE);
}

sp_application_crypto::with_pair! {
	/// The BEEFY crypto scheme defined via the keypair type.
	pub type AuthorityPair = app::Pair;
}

/// Identity of a BEEFY authority.
pub type AuthorityId = app::Public;

/// Signature for a BEEFY authority.
pub type AuthoritySignature = app::Signature;

/// Allows all gossip messages to get through.
struct AllowAll<Hash> {
	topic: Hash,
}

impl<Block> GossipValidator<Block> for AllowAll<Block::Hash>
where
	Block: BlockT,
{
	fn validate(
		&self,
		_context: &mut dyn GossipValidatorContext<Block>,
		_sender: &sc_network::PeerId,
		_data: &[u8],
	) -> GossipValidationResult<Block::Hash> {
		GossipValidationResult::ProcessAndKeep(self.topic)
	}
}

struct RoundTracker<Id, Signature> {
	votes: Vec<(Id, Signature)>,
}

impl<Id, Signature> Default for RoundTracker<Id, Signature> {
	fn default() -> Self {
		RoundTracker {
			votes: Vec::new(),
		}
	}
}

impl<Id, Signature> RoundTracker<Id, Signature>
where
	Id: PartialEq,
	Signature: PartialEq,
{
	fn add_vote(&mut self, vote: (Id, Signature)) -> bool {
		// this needs to handle equivocations in the future
		if self.votes.contains(&vote) {
			return false;
		}

		self.votes.push(vote);
		true
	}

	fn is_done(&self, threshold: usize) -> bool {
		self.votes.len() >= threshold
	}
}

fn threshold(voters: usize) -> usize {
	let faulty = voters.saturating_sub(1) / 3;
	voters - faulty
}

struct Rounds<Hash, Id, Signature> {
	rounds: BTreeMap<Hash, RoundTracker<Id, Signature>>,
	voters: Vec<Id>,
}

impl<Hash, Id, Signature> Rounds<Hash, Id, Signature>
where
	Hash: Ord,
{
	fn new(voters: Vec<Id>) -> Self {
		Rounds {
			rounds: BTreeMap::new(),
			voters,
		}
	}
}

impl<Hash, Id, Signature> Rounds<Hash, Id, Signature>
where
	Hash: Ord,
	Id: PartialEq,
	Signature: PartialEq,
{
	fn add_vote(&mut self, round: Hash, vote: (Id, Signature)) -> bool {
		self.rounds.entry(round).or_default().add_vote(vote)
	}

	fn is_done(&self, round: &Hash) -> bool {
		self.rounds
			.get(round)
			.map(|tracker| tracker.is_done(threshold(self.voters.len())))
			.unwrap_or(false)
	}

	fn drop(&mut self, round: &Hash) {
		self.rounds.remove(round);
	}
}

struct BeefyWorker<Block: BlockT, Id, Signature, FinalityNotifications> {
	rounds: Rounds<Block::Hash, Id, Signature>,
	finality_notifications: FinalityNotifications,
	gossip_engine: GossipEngine<Block>,
}

impl<Block, Id, Signature, FinalityNotifications>
	BeefyWorker<Block, Id, Signature, FinalityNotifications>
where
	Block: BlockT,
{
	fn new(
		voters: Vec<Id>,
		finality_notifications: FinalityNotifications,
		gossip_engine: GossipEngine<Block>,
	) -> Self {
		BeefyWorker {
			rounds: Rounds::new(voters),
			finality_notifications,
			gossip_engine,
		}
	}
}

fn topic<Block: BlockT>() -> Block::Hash {
	<<Block::Header as HeaderT>::Hashing as HashT>::hash("beefy".as_bytes())
}

#[derive(Decode, Encode)]
struct VoteMessage<Hash, Id, Signature> {
	block: Hash,
	id: Id,
	signature: Signature,
}

impl<Block, Id, Signature, FinalityNotifications>
	BeefyWorker<Block, Id, Signature, FinalityNotifications>
where
	Block: BlockT,
	Id: Codec + PartialEq,
	Signature: Codec + PartialEq,
	FinalityNotifications: Stream<Item = FinalityNotification<Block>> + Unpin,
{
	fn handle_finality_notification(&mut self, notification: FinalityNotification<Block>) {
		info!(target: "beefy", "Finality notification: {:?}", notification);
	}

	fn handle_vote(&mut self, round: Block::Hash, vote: (Id, Signature)) {
		if self.rounds.add_vote(round.clone(), vote) {
			if self.rounds.is_done(&round) {
				info!(target: "beefy", "Round {:?} concluded.", round);
				self.rounds.drop(&round);
			}
		}
	}

	async fn run(mut self) {
		let mut votes = Box::pin(self.gossip_engine.messages_for(topic::<Block>()).filter_map(
			|notification| async move {
				VoteMessage::<Block::Hash, Id, Signature>::decode(&mut &notification.message[..])
					.ok()
			},
		));

		loop {
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
						self.handle_vote(vote.block, (vote.id, vote.signature));
					} else {
						return;
					}
				}
			}
		}
	}
}

pub async fn start_beefy_gadget<Block, Backend, Client, Network, SyncOracle>(
	client: Arc<Client>,
	network: Network,
	_sync_oracle: SyncOracle,
) where
	Block: BlockT,
	Backend: BackendT<Block>,
	Client: BlockchainEvents<Block> + Finalizer<Block, Backend> + Send + Sync,
	Network: GossipNetwork<Block> + Clone + Send + 'static,
	SyncOracle: SyncOracleT + Send + 'static,
{
	let gossip_engine = GossipEngine::new(
		network,
		BEEFY_ENGINE_ID,
		BEEFY_PROTOCOL_NAME,
		Arc::new(AllowAll {
			topic: topic::<Block>(),
		}),
	);

	let voters = vec![];

	let worker = BeefyWorker::<_, AuthorityId, AuthoritySignature, _>::new(
		voters,
		client.finality_notification_stream(),
		gossip_engine,
	);

	worker.run().await
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
