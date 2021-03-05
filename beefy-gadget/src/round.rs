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

use std::collections::BTreeMap;

struct RoundTracker<Id, Signature> {
	votes: Vec<(Id, Signature)>,
}

impl<Id, Signature> Default for RoundTracker<Id, Signature> {
	fn default() -> Self {
		RoundTracker { votes: Vec::new() }
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

fn threshold(authorities: usize) -> usize {
	let faulty = authorities.saturating_sub(1) / 3;
	authorities - faulty
}

pub(crate) struct Rounds<Hash, Number, Id, Signature> {
	rounds: BTreeMap<(Hash, Number), RoundTracker<Id, Signature>>,
	authorities: Vec<Id>,
}

impl<Hash, Number, Id, Signature> Rounds<Hash, Number, Id, Signature>
where
	Hash: Ord,
	Number: Ord,
{
	pub(crate) fn new(authorities: Vec<Id>) -> Self {
		Rounds {
			rounds: BTreeMap::new(),
			authorities,
		}
	}
}

impl<Hash, Number, Id, Signature> Rounds<Hash, Number, Id, Signature>
where
	Hash: Ord,
	Number: Ord,
	Id: PartialEq,
	Signature: Clone + PartialEq,
{
	pub(crate) fn add_vote(&mut self, round: (Hash, Number), vote: (Id, Signature)) -> bool {
		self.rounds.entry(round).or_default().add_vote(vote)
	}

	pub(crate) fn is_done(&self, round: &(Hash, Number)) -> bool {
		self.rounds
			.get(round)
			.map(|tracker| tracker.is_done(threshold(self.authorities.len())))
			.unwrap_or(false)
	}

	pub(crate) fn drop(&mut self, round: &(Hash, Number)) -> Option<Vec<Option<Signature>>> {
		let signatures = self.rounds.remove(round)?.votes;

		Some(
			self.authorities
				.iter()
				.map(|authority_id| {
					signatures
						.iter()
						.find_map(|(id, sig)| if id == authority_id { Some(sig.clone()) } else { None })
				})
				.collect(),
		)
	}
}
