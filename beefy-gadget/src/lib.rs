use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use futures::{FutureExt, Stream, StreamExt};
use log::info;

use sc_client_api::{Backend as BackendT, BlockchainEvents, FinalityNotification, Finalizer};
use sc_network_gossip::Network as GossipNetwork;
use sp_consensus::SyncOracle as SyncOracleT;
use sp_runtime::traits::Block as BlockT;

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
}

impl<Block, Id, Signature, FinalityNotifications>
	BeefyWorker<Block, Id, Signature, FinalityNotifications>
where
	Block: BlockT,
{
	fn new(voters: Vec<Id>, finality_notifications: FinalityNotifications) -> Self {
		BeefyWorker {
			rounds: Rounds::new(voters),
			finality_notifications,
		}
	}
}

impl<Block, Id, Signature, FinalityNotifications>
	BeefyWorker<Block, Id, Signature, FinalityNotifications>
where
	Block: BlockT,
	Id: PartialEq,
	Signature: PartialEq,
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
		loop {
			futures::select! {
				notification = self.finality_notifications.next().fuse() => {
					if let Some(notification) = notification {
						self.handle_finality_notification(notification);
					} else {
						return;
					}
				},
			}
		}
	}
}

pub async fn start_beefy_gadget<Block, Backend, Client, Network, SyncOracle>(
	client: Arc<Client>,
	_network: Network,
	_sync_oracle: SyncOracle,
) where
	Block: BlockT,
	Backend: BackendT<Block>,
	Client: BlockchainEvents<Block> + Finalizer<Block, Backend> + Send + Sync,
	Network: GossipNetwork<Block> + Clone + Send + 'static,
	SyncOracle: SyncOracleT + Send + 'static,
{
	type Id = usize;
	type Signature = ();

	let mut worker =
		BeefyWorker::<Block, Id, Signature, _>::new(vec![], client.finality_notification_stream());

	worker.run().await
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
