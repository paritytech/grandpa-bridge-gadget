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

use crate::cli::utils::Bytes;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use structopt::StructOpt;

/// MMR related commands
#[derive(StructOpt)]
#[structopt(about = "Merkle Mountain Range related commands.")]
pub enum Mmr {
	/// Decode Polkadot-compatible MMR Leaf.
	DecodeLeaf {
		/// A double SCALE-encoded MMR Leaf.
		///
		/// Leaf can be obtained via `mmr_generateProof` custom RPC method.
		/// Since the RPC returns a SCALE-encoding of `Vec<u8>`, this method expects the same.
		leaf: Bytes,
	},
	/// Construct MMR Offchain storage key.
	StorageKey {
		/// Indexing prefix used in pallet configuration.
		prefix: String,
		/// Node position.
		pos: u64,
	},
}

/// A MMR leaf structure (should be matching one in Polkadot repo).
#[derive(Debug, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct MmrLeaf {
	/// Current block parent number and hash.
	pub parent_number_and_hash: (u32, H256),
	/// A merkle root of all registered parachain heads.
	pub parachain_heads: H256,
	/// A merkle root of the next BEEFY authority set.
	pub beefy_next_authority_set: BeefyNextAuthoritySet,
}

/// Details of the next BEEFY authority set.
#[derive(Debug, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct BeefyNextAuthoritySet {
	/// Id of the next set.
	pub id: u64,
	/// Number of validators in the set.
	pub len: u32,
	/// Merkle Root Hash build from BEEFY uncompressed AuthorityIds.
	pub root: H256,
}

impl Mmr {
	pub fn run(self) -> anyhow::Result<()> {
		match self {
			Self::DecodeLeaf { leaf } => {
				let leaf: Vec<u8> = Decode::decode(&mut &*leaf.0)?;
				let leaf: MmrLeaf = Decode::decode(&mut &*leaf)?;
				println!("{:?}", leaf);
			}
			Self::StorageKey { prefix, pos } => {
				let key = (prefix.as_bytes(), pos).encode();
				println!("0x{}", hex::encode(&key));
			}
		}
		Ok(())
	}
}
