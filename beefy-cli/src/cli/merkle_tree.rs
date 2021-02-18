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

use sp_core::H256;
use crate::cli::utils::{Authorities, Bytes};
use structopt::StructOpt;
use parity_scale_codec::{Encode, Decode};

/// BEEFY authority id merkle tree related commands.
#[derive(StructOpt)]
#[structopt(about = "Construct or verify a merkle proof from BEEFY authorities.")]
pub enum BeefyMerkleTree {
	/// Construct a merkle tree of uncompressed public keys, given BEEFY authority ids (compressed
	/// keys) and generate a merkle proof.
	GenerateProof {
		/// A SCALE-encoded vector of BEEFY authority ids (compressed public key).
		authorities: Authorities,
		/// Leaf index to generate the proof for.
		leaf_index: usize,
	},
	/// Verify a merkle proof given root hash and the proof content.
	VerifyProof {
		/// Merkle Trie Root hash.
		root: H256,
		/// Proof content.
		proof: Bytes,
	}
}

impl BeefyMerkleTree {
	pub fn run(self) -> anyhow::Result<()> {
		match self {
			Self::GenerateProof { authorities, leaf_index } => {
				generate_merkle_proof(authorities.0, leaf_index)
			},
			Self::VerifyProof { root, proof } => {
				unimplemented!()
			}
		}
	}
}

/// Parachain heads merkle tree related commands.
#[derive(StructOpt)]
#[structopt(about = "Construct or verify a merkle proof from parachain heads.")]
pub enum ParaMerkleTree {}

impl ParaMerkleTree {
	pub fn run(self) -> anyhow::Result<()> {
		unimplemented!()
	}
}


fn generate_merkle_proof<T: Encode>(
	items: Vec<T>,
	leaf_index: usize,
) -> anyhow::Result<()> {
	use sp_trie::TrieConfiguration;

	type Layout = sp_trie::Layout<sp_core::KeccakHasher>;

	let items = items.iter().map(Encode::encode).collect::<Vec<_>>();
	let ordered_items = items
		.iter()
		.enumerate()
		.map(|(i, v)| (Layout::encode_index(i as u32), v.clone()))
		.collect::<Vec<(Vec<u8>, Vec<u8>)>>();
	let mut db = sp_trie::MemoryDB::<sp_core::KeccakHasher>::default();
	let mut cb = trie_db::TrieBuilder::new(&mut db);
	trie_db::trie_visit::<Layout, _, _, _, _>(ordered_items.into_iter(), &mut cb);
	let root = cb.root.unwrap_or_default();

	let proof = sp_trie::generate_trie_proof::<Layout, _, _, _>(
		&db,
		root,
		vec![&Layout::encode_index(leaf_index as u32)],
	)?;

	println!("Root: {:?}", root);
	println!("SCALE-encoded proof: 0x{}", hex::encode(proof.encode()));

	Ok(())
}
