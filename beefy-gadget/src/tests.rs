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

#![allow(dead_code)]

use sc_network::config::ProtocolConfig;
use sc_network_test::{PassThroughVerifier, Peer, PeersClient, TestNetFactory};

use beefy_primitives::{ecdsa::AuthorityId, ValidatorSet};

struct MockNetwork {
	peers: Vec<Peer<()>>,
	validator_set: ValidatorSet<AuthorityId>,
}

impl MockNetwork {
	fn new(validator_set: ValidatorSet<AuthorityId>, num_peers: usize) -> Self {
		let mut net = MockNetwork {
			peers: Vec::with_capacity(num_peers),
			validator_set,
		};

		for _ in 0..num_peers {
			net.add_full_peer();
		}

		net
	}
}

impl TestNetFactory for MockNetwork {
	type Verifier = PassThroughVerifier;
	type PeerData = ();

	fn from_config(_config: &ProtocolConfig) -> Self {
		MockNetwork {
			peers: Vec::new(),
			validator_set: Default::default(),
		}
	}

	fn make_verifier(
		&self,
		_client: PeersClient,
		_config: &ProtocolConfig,
		_peer_data: &Self::PeerData,
	) -> Self::Verifier {
		PassThroughVerifier::new(false)
	}

	fn peer(&mut self, i: usize) -> &mut Peer<Self::PeerData> {
		&mut self.peers[i]
	}

	fn peers(&self) -> &Vec<Peer<Self::PeerData>> {
		&self.peers
	}

	fn mut_peers<F: FnOnce(&mut Vec<Peer<Self::PeerData>>)>(&mut self, closure: F) {
		closure(&mut self.peers)
	}
}
