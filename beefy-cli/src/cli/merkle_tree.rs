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

use crate::cli::{
	uncompress_authorities::uncompress_beefy_ids,
	utils::{Authorities, Bytes},
};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use structopt::StructOpt;

/// BEEFY authority id merkle tree related commands.
#[derive(StructOpt)]
#[structopt(about = "Construct or verify a merkle proof from BEEFY authorities.")]
pub enum BeefyMerkleTree {
	/// Construct a merkle tree of uncompressed public keys, given BEEFY authority ids (compressed
	/// keys) and generate a merkle proof.
	GenerateProof {
		/// Leaf index to generate the proof for.
		leaf_index: usize,
		/// A SCALE-encoded vector of BEEFY authority ids (compressed public key).
		authorities: Authorities,
	},
	/// Verify a merkle proof given root hash and the proof content.
	VerifyProof {
		/// Merkle Trie Root hash.
		root: H256,
		/// Proof content.
		proof: Bytes,
		/// Index of the leaf the proof is for.
		leaf_index: usize,
		/// SCALE-encoded value of the leaf node (it's not part of the proof).
		leaf_value: Bytes,
	},
}

impl BeefyMerkleTree {
	pub fn run(self) -> anyhow::Result<()> {
		match self {
			Self::GenerateProof {
				authorities,
				leaf_index,
			} => {
				let uncompressed = uncompress_beefy_ids(authorities.0)?;
				let uncompressed_raw = uncompressed.into_iter().map(|k| k.serialize());
				print_generated_merkle_proof(uncompressed_raw, leaf_index)
			}
			Self::VerifyProof {
				root,
				proof,
				leaf_index,
				leaf_value,
			} => verify_merkle_proof(root, proof.0, leaf_index, leaf_value.0),
		}
	}
}

/// Parachain heads merkle tree related commands.
#[derive(StructOpt)]
#[structopt(about = "Construct or verify a merkle proof from parachain heads.")]
pub enum ParaMerkleTree {
	/// Construct a merkle tree of given list of parachains' `HeadData`
	/// and generate a merkle proof.
	GenerateProof {
		/// Leaf index to generate the proof for.
		leaf_index: usize,
		/// A list of raw `HeadData`.
		heads: Vec<Bytes>,
	},
	/// Verify a merkle proof given root hash and the proof content.
	VerifyProof {
		/// Merkle Trie Root hash.
		root: H256,
		/// Proof content.
		proof: Bytes,
		/// Index of the leaf the proof is for.
		leaf_index: usize,
		/// SCALE-encoded value of the leaf node (it's not part of the proof).
		leaf_value: Bytes,
	},
}

impl ParaMerkleTree {
	pub fn run(self) -> anyhow::Result<()> {
		match self {
			Self::GenerateProof { heads, leaf_index } => {
				let raw_heads = heads.into_iter().map(|x| x.0);
				print_generated_merkle_proof(raw_heads, leaf_index)
			}
			Self::VerifyProof {
				root,
				proof,
				leaf_index,
				leaf_value,
			} => verify_merkle_proof(root, proof.0, leaf_index, leaf_value.0),
		}
	}
}

use sp_trie::{TrieConfiguration, TrieMut};

type Proof = Vec<Vec<u8>>;
type Layout = sp_trie::Layout<sp_core::KeccakHasher>;
type Leaf = (Vec<u8>, Vec<u8>);

fn generate_merkle_proof<T: Encode>(
	items: impl Iterator<Item = T>,
	leaf_index: usize,
) -> anyhow::Result<(H256, Proof, Leaf)> {
	let ordered_items = items
		.map(|x| Encode::encode(&x))
		.enumerate()
		.map(|(i, v)| (Layout::encode_index(i as u32), v))
		.collect::<Vec<(Vec<u8>, Vec<u8>)>>();
	let leaf = ordered_items
		.get(leaf_index)
		.cloned()
		.ok_or_else(|| anyhow::format_err!("Leaf index out of bounds: {} vs {}", leaf_index, ordered_items.len(),))?;
	let mut db = sp_trie::MemoryDB::<sp_core::KeccakHasher>::default();
	let mut root = sp_trie::empty_trie_root::<Layout>();
	{
		let mut trie = sp_trie::TrieDBMut::<Layout>::new(&mut db, &mut root);
		for (k, v) in ordered_items {
			trie.insert(&k, &v).unwrap();
		}
	}
	let proof: Proof = sp_trie::generate_trie_proof::<Layout, _, _, _>(&db, root, vec![&leaf.0])?;

	Ok((root, proof, leaf))
}

fn print_generated_merkle_proof<T: Encode>(items: impl Iterator<Item = T>, leaf_index: usize) -> anyhow::Result<()> {
	let (root, proof, leaf) = generate_merkle_proof(items, leaf_index)?;
	println!();
	println!("Root: {:?}", root);
	println!("SCALE-encoded proof: 0x{}", hex::encode(proof.encode()));
	println!("\nLeaf key: 0x{}", hex::encode(&leaf.0));
	println!("SCALE-encoded leaf value: 0x{}", hex::encode(&leaf.1));
	println!();

	Ok(())
}

fn verify_merkle_proof(root: H256, proof: Vec<u8>, leaf_index: usize, leaf_value: Vec<u8>) -> anyhow::Result<()> {
	let proof: Proof = Decode::decode(&mut &*proof)?;
	let items = vec![(Layout::encode_index(leaf_index as u32), Some(leaf_value))];

	sp_trie::verify_trie_proof::<Layout, _, _, _>(&root, &*proof, items.iter())?;

	println!("\nProof is correct.\n");

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;
	use sp_core::crypto::UncheckedInto;

	#[test]
	fn generate_proof_should_be_verified_correctly() {
		// given
		let authorities = Authorities(vec![
			hex!("039346ec0021405ec103c2baac8feff9d6fb75851318fb03781edf29f05f2ffeb7").unchecked_into(),
			hex!("03fe6b333420b90689158643ccad94e62d707de1a80726d53aa04657fec14afd3e").unchecked_into(),
			hex!("03fe6b333420b90689158643ccad94e62d707de1a80726d53aa04657fec14afd3e").unchecked_into(),
		]);
		let uncompressed = uncompress_beefy_ids(authorities.0).unwrap();
		let items = uncompressed.into_iter().map(|k| k.serialize());
		let leaf_index = 0;

		// when
		let (root, proof, leaf) = generate_merkle_proof(items, leaf_index).unwrap();

		// then
		verify_merkle_proof(root, proof.encode(), leaf_index, leaf.1).unwrap();
	}
}
