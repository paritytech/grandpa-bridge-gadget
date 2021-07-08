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
	uncompress_authorities::{uncompress_beefy_ids, uncompressed_to_eth},
	utils::{Authorities, Bytes},
};
use beefy_merkle_tree::Keccak256;
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
		/// Number of leaves in the original tree.
		number_of_leaves: usize,
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
				let eth_addresses = uncompressed_to_eth(uncompressed);
				print_generated_merkle_proof(eth_addresses, leaf_index)
			}
			Self::VerifyProof {
				root,
				proof,
				number_of_leaves,
				leaf_index,
				leaf_value,
			} => verify_merkle_proof(root, proof.0, number_of_leaves, leaf_index, leaf_value.0),
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
		heads: Vec<Bytes>, // TODO [ToDr] Add ParaId
	},
	/// Verify a merkle proof given root hash and the proof content.
	VerifyProof {
		/// Merkle Trie Root hash.
		root: H256,
		/// Proof content.
		proof: Bytes,
		/// Number of leaves in the original tree.
		number_of_leaves: usize,
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
				number_of_leaves,
				leaf_index,
				leaf_value,
			} => verify_merkle_proof(root, proof.0, number_of_leaves, leaf_index, leaf_value.0),
		}
	}
}

type Proof = Vec<H256>;
type Leaf = Vec<u8>;

fn generate_merkle_proof<T: AsRef<[u8]>>(
	items: impl Iterator<Item = T>,
	leaf_index: usize,
) -> anyhow::Result<(H256, Proof, Leaf, usize)> {
	let items = items.collect::<Vec<_>>();
	let number_of_leaves = items.len();
	let leaf = items
		.get(leaf_index)
		.map(|x| x.as_ref().to_vec())
		.ok_or_else(|| anyhow::format_err!("Leaf index out of bounds: {} vs {}", leaf_index, items.len(),))?;

	let beefy_merkle_tree::MerkleProof { root, proof, .. } =
		beefy_merkle_tree::merkle_proof::<Keccak256, _, _>(items, leaf_index);
	let proof = proof.into_iter().map(Into::into).collect();

	Ok((root.into(), proof, leaf, number_of_leaves))
}

fn print_generated_merkle_proof<T: AsRef<[u8]>>(
	items: impl Iterator<Item = T>,
	leaf_index: usize,
) -> anyhow::Result<()> {
	let (root, proof, leaf, number_of_leaves) = generate_merkle_proof(items, leaf_index)?;
	println!();
	println!("Root: {:?}", root);
	println!("Leaf index: {}", leaf_index);
	println!("Number of leaves: {}", number_of_leaves);
	println!("SCALE-encoded proof: 0x{}", hex::encode(proof.encode()));
	println!("SCALE-encoded leaf value: 0x{}", hex::encode(&leaf));
	println!();

	Ok(())
}

fn verify_merkle_proof(
	root: H256,
	proof: Vec<u8>,
	number_of_leaves: usize,
	leaf_index: usize,
	leaf_value: Vec<u8>,
) -> anyhow::Result<()> {
	let proof: Proof = Decode::decode(&mut &*proof)?;
	let convert = |c: H256| c.to_fixed_bytes();
	let root = convert(root);
	let proof = proof.into_iter().map(convert).collect::<Vec<_>>();

	if beefy_merkle_tree::verify_proof::<Keccak256, _, _>(&root, proof, number_of_leaves, leaf_index, &leaf_value) {
		println!("\n✅ Proof is correct.\n");
	} else {
		println!("\n❌ Proof is INCORRECT.\n");
	}

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
		let len = authorities.0.len();
		let uncompressed = uncompress_beefy_ids(authorities.0).unwrap();
		let items = uncompressed_to_eth(uncompressed);
		let leaf_index = 0;

		// when
		let (root, proof, leaf, _) = generate_merkle_proof(items, leaf_index).unwrap();

		// then
		verify_merkle_proof(root, proof.encode(), len, leaf_index, leaf).unwrap();
	}
}
