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

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! This crate implements a simple binary Merkle Tree utilities required for inter-op with Ethereum
//! bridge & Solidity contract.
//!
//! The implementation is optimised for usage within Substrate Runtime and supports no-std
//! compilation targets.
//!
//! Merkle Tree is constructed from arbitrary-length leaves, that are initially hashed using the
//! same [Hasher] as the inner nodes.
//! Inner nodes are created by concatenating child hashes and hashing again. The implementation
//! does not perform any sorting of the input data (leaves) nor when inner nodes are created.
//!
//! If the number of leaves is not even, last leave (hash of) is promoted to the upper layer.

#[cfg(not(feature = "std"))]
use core::vec::Vec;

/// Supported hashing output size.
///
/// The size is restricted to 32 bytes to allow for a more optimised implementation.
pub type Hash = [u8; 32];

/// Generic hasher trait.
///
/// Implement the function to support custom way of hashing data.
/// The implementation must return a [Hash] type, so only 32-byte output hashes are supported.
pub trait Hasher {
	/// Hash given arbitrary-length piece of data.
	fn hash(data: &[u8]) -> Hash;
}

#[cfg(feature = "keccak")]
mod keccak256 {
	use tiny_keccak::{Hasher as _, Keccak};

	/// Keccak256 hasher implementation.
	pub struct Keccak256;
	impl Keccak256 {
		/// Hash given data.
		pub fn hash(data: &[u8]) -> super::Hash {
			<Keccak256 as super::Hasher>::hash(data)
		}
	}
	impl super::Hasher for Keccak256 {
		fn hash(data: &[u8]) -> super::Hash {
			let mut keccak = Keccak::v256();
			keccak.update(data);
			let mut output = [0_u8; 32];
			keccak.finalize(&mut output);
			output
		}
	}
}
#[cfg(feature = "keccak")]
pub use keccak256::Keccak256;

/// Construct a root hash of a Binary Merkle Tree created from given leaves.
///
/// See crate-level docs for details about Merkle Tree construction.
///
/// In case an empty list of leaves is passed the function returns a 0-filled hash.
pub fn merkle_root<H, I, T>(leaves: I) -> Hash
where
	H: Hasher,
	I: IntoIterator<Item = T>,
	T: AsRef<[u8]>,
{
	let iter = leaves.into_iter().map(|l| H::hash(l.as_ref()));
	merkelize::<H, _, _>(iter, &mut ())
}

fn merkelize<H, V, I>(leaves: I, visitor: &mut V) -> Hash
where
	H: Hasher,
	V: Visitor,
	I: Iterator<Item = Hash>,
{
	let upper = Vec::with_capacity(leaves.size_hint().0);
	let mut next = match merkelize_row::<H, _, _>(leaves, upper, visitor) {
		Ok(root) => return root,
		Err(next) if next.is_empty() => return Hash::default(),
		Err(next) => next,
	};

	let mut upper = Vec::with_capacity((next.len() + 1) / 2);
	loop {
		visitor.move_up();

		match merkelize_row::<H, _, _>(next.drain(..), upper, visitor) {
			Ok(root) => return root,
			Err(t) => {
				// swap collections to avoid allocations
				upper = next;
				next = t;
			}
		};
	}
}

/// A generated merkle proof.
///
/// The structure contains all necessary data to later on verify the proof and the leaf itself.
pub struct MerkleProof<T> {
	/// Root hash of generated merkle tree.
	pub root: Hash,
	/// Proof items (does not contain the leaf hash, nor the root obviously).
	///
	/// This vec contains all inner node hashes necessary to reconstruct the root hash given the
	/// leaf hash.
	pub proof: Vec<Hash>,
	/// Number of leaves in the original tree.
	///
	/// This is needed to detect a case where we have an odd number of leaves that "get promoted"
	/// to upper layers.
	pub number_of_leaves: usize,
	/// Index of the leaf the proof is for.
	pub leaf_index: usize,
	/// Leaf content.
	pub leaf: T,
}

/// A trait of object inspecting merkle root creation.
///
/// It can be passed to [`merkelize_row`] or [`merkelize`] functions and will be notified
/// about tree traversal.
trait Visitor {
	/// We are moving one level up in the tree.
	fn move_up(&mut self);

	/// We are creating an inner node from given `left` and `right` nodes.
	///
	/// Note that in case of last odd node in the row `right` might be empty.
	/// The method will also visit the `root` hash (level 0).
	///
	/// The `index` is an index of `left` item.
	fn visit(&mut self, index: usize, left: &Option<Hash>, right: &Option<Hash>);
}

/// No-op implementation of the visitor.
impl Visitor for () {
	fn move_up(&mut self) {}
	fn visit(&mut self, _index: usize, _left: &Option<Hash>, _right: &Option<Hash>) {}
}

/// Construct a Merkle Proof for leaves given by indices.
///
/// The function constructs a (partial) Merkle Tree first and stores all elements required
/// to prove requested item (leaf) given the root hash.
///
/// Both the Proof and the Root Hash is returned.
///
/// # Panic
///
/// The function will panic if given [`leaf_index`] is greater than the number of leaves.
pub fn merkle_proof<H, I, T>(leaves: I, leaf_index: usize) -> MerkleProof<T>
where
	H: Hasher,
	I: IntoIterator<Item = T>,
	I::IntoIter: ExactSizeIterator,
	T: AsRef<[u8]>,
{
	let mut leaf = None;
	let iter = leaves.into_iter().enumerate().map(|(idx, l)| {
		let hash = H::hash(l.as_ref());
		if idx == leaf_index {
			leaf = Some(l);
		}
		hash
	});

	struct ProofCollection {
		proof: Vec<Hash>,
		position: usize,
	}

	impl ProofCollection {
		fn new(position: usize) -> Self {
			ProofCollection {
				proof: Default::default(),
				position,
			}
		}
	}

	impl Visitor for ProofCollection {
		fn move_up(&mut self) {
			self.position /= 2;
		}

		fn visit(&mut self, index: usize, left: &Option<Hash>, right: &Option<Hash>) {
			// we are at left branch - right goes to the proof.
			if self.position == index {
				if let Some(right) = right {
					self.proof.push(*right);
				}
			}
			// we are at right branch - left goes to the proof.
			if self.position == index + 1 {
				if let Some(left) = left {
					self.proof.push(*left);
				}
			}
		}
	}

	let number_of_leaves = iter.len();
	let mut collect_proof = ProofCollection::new(leaf_index);

	let root = merkelize::<H, _, _>(iter, &mut collect_proof);
	let leaf = leaf.expect("Requested `leaf_index` is greater than number of leaves.");

	#[cfg(feature = "debug")]
	log::debug!(
		"[merkle_proof] Proof: {:?}",
		collect_proof.proof.iter().map(hex::encode).collect::<Vec<_>>()
	);

	MerkleProof {
		root,
		proof: collect_proof.proof,
		number_of_leaves,
		leaf_index,
		leaf,
	}
}

/// Leaf node for proof verification.
///
/// Can be either a value that needs to be hashed first,
/// or the hash itself.
#[derive(Debug, PartialEq, Eq)]
pub enum Leaf<'a> {
	/// Leaf content.
	Value(&'a [u8]),
	/// Hash of the leaf content.
	Hash(Hash),
}

impl<'a, T: AsRef<[u8]>> From<&'a T> for Leaf<'a> {
	fn from(v: &'a T) -> Self {
		Leaf::Value(v.as_ref())
	}
}

impl<'a> From<Hash> for Leaf<'a> {
	fn from(v: Hash) -> Self {
		Leaf::Hash(v)
	}
}

/// Verify Merkle Proof correctness versus given root hash.
///
/// The proof is NOT expected to contain leaf hash as the first
/// element, but only all adjacent nodes required to eventually by process of
/// concatenating and hashing end up with given root hash.
///
/// The proof must not contain the root hash.
pub fn verify_proof<'a, H, P, L>(root: &'a Hash, proof: P, number_of_leaves: usize, leaf_index: usize, leaf: L) -> bool
where
	H: Hasher,
	P: IntoIterator<Item = Hash>,
	L: Into<Leaf<'a>>,
{
	if leaf_index >= number_of_leaves {
		return false;
	}

	let leaf_hash = match leaf.into() {
		Leaf::Value(content) => H::hash(content),
		Leaf::Hash(hash) => hash,
	};

	let mut combined = [0_u8; 64];
	let mut position = leaf_index;
	let mut width = number_of_leaves;
	let computed = proof.into_iter().fold(leaf_hash, |a, b| {
		if position % 2 == 1 || position + 1 == width {
			combined[0..32].copy_from_slice(&b);
			combined[32..64].copy_from_slice(&a);
		} else {
			combined[0..32].copy_from_slice(&a);
			combined[32..64].copy_from_slice(&b);
		}
		let hash = H::hash(&combined);
		#[cfg(feature = "debug")]
		log::debug!(
			"[verify_proof]: (a, b) {:?}, {:?} => {:?} ({:?}) hash",
			hex::encode(a),
			hex::encode(b),
			hex::encode(hash),
			hex::encode(combined)
		);
		position /= 2;
		width = ((width - 1) / 2) + 1;
		hash
	});

	root == &computed
}

/// Processes a single row (layer) of a tree by taking pairs of elements,
/// concatenating them, hashing and placing into resulting vector.
///
/// In case only one element is provided it is returned via `Ok` result, in any other case (also an
/// empty iterator) an `Err` with the inner nodes of upper layer is returned.
fn merkelize_row<H, V, I>(mut iter: I, mut next: Vec<Hash>, visitor: &mut V) -> Result<Hash, Vec<Hash>>
where
	H: Hasher,
	V: Visitor,
	I: Iterator<Item = Hash>,
{
	#[cfg(feature = "debug")]
	log::debug!("[merkelize_row]");
	next.clear();

	let mut index = 0;
	let mut combined = [0_u8; 64];
	loop {
		let a = iter.next();
		let b = iter.next();
		visitor.visit(index, &a, &b);

		#[cfg(feature = "debug")]
		log::debug!(
			"  {:?}\n  {:?}",
			a.as_ref().map(hex::encode),
			b.as_ref().map(hex::encode)
		);

		index += 2;
		match (a, b) {
			(Some(a), Some(b)) => {
				combined[0..32].copy_from_slice(&a);
				combined[32..64].copy_from_slice(&b);

				next.push(H::hash(&combined));
			}
			// Odd number of items. Promote the item to the upper layer.
			(Some(a), None) if !next.is_empty() => {
				next.push(a);
			}
			// Last item = root.
			(Some(a), None) => {
				return Ok(a);
			}
			// Finish up, no more items.
			_ => {
				#[cfg(feature = "debug")]
				log::debug!(
					"[merkelize_row] Next: {:?}",
					next.iter().map(hex::encode).collect::<Vec<_>>()
				);
				return Err(next);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	#[test]
	fn should_generate_empty_root() {
		// given
		let _ = env_logger::try_init();
		let data: Vec<[u8; 1]> = Default::default();

		// when
		let out = merkle_root::<Keccak256, _, _>(data);

		// then
		assert_eq!(
			hex::encode(&out),
			"0000000000000000000000000000000000000000000000000000000000000000"
		);
	}

	#[test]
	fn should_generate_single_root() {
		// given
		let _ = env_logger::try_init();
		let data = vec![hex!("E04CC55ebEE1cBCE552f250e85c57B70B2E2625b")];

		// when
		let out = merkle_root::<Keccak256, _, _>(data);

		// then
		assert_eq!(
			hex::encode(&out),
			"aeb47a269393297f4b0a3c9c9cfd00c7a4195255274cf39d83dabc2fcc9ff3d7"
		);
	}

	#[test]
	fn should_generate_root_pow_2() {
		// given
		let _ = env_logger::try_init();
		let data = vec![
			hex!("E04CC55ebEE1cBCE552f250e85c57B70B2E2625b"),
			hex!("25451A4de12dcCc2D166922fA938E900fCc4ED24"),
		];

		// when
		let out = merkle_root::<Keccak256, _, _>(data);

		// then
		assert_eq!(
			hex::encode(&out),
			"697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402"
		);
	}

	#[test]
	fn should_generate_root_complex() {
		let _ = env_logger::try_init();
		let test = |root, data| {
			assert_eq!(hex::encode(&merkle_root::<Keccak256, _, _>(data)), root);
		};

		test(
			"aff1208e69c9e8be9b584b07ebac4e48a1ee9d15ce3afe20b77a4d29e4175aa3",
			vec!["a", "b", "c"],
		);

		test(
			"b8912f7269068901f231a965adfefbc10f0eedcfa61852b103efd54dac7db3d7",
			vec!["a", "b", "a"],
		);

		test(
			"dc8e73fe6903148ff5079baecc043983625c23b39f31537e322cd0deee09fa9c",
			vec!["a", "b", "a", "b"],
		);

		test(
			"fb3b3be94be9e983ba5e094c9c51a7d96a4fa2e5d8e891df00ca89ba05bb1239",
			vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
		);
	}

	#[test]
	fn should_generate_and_verify_proof_simple() {
		// given
		let _ = env_logger::try_init();
		let data = vec!["a", "b", "c"];

		// when
		let proof0 = merkle_proof::<Keccak256, _, _>(data.clone(), 0);
		assert!(verify_proof::<Keccak256, _, _>(
			&proof0.root,
			proof0.proof.clone(),
			data.len(),
			proof0.leaf_index,
			&proof0.leaf,
		));

		let proof1 = merkle_proof::<Keccak256, _, _>(data.clone(), 1);
		assert!(verify_proof::<Keccak256, _, _>(
			&proof1.root,
			proof1.proof,
			data.len(),
			proof1.leaf_index,
			&proof1.leaf,
		));

		let proof2 = merkle_proof::<Keccak256, _, _>(data.clone(), 2);
		assert!(verify_proof::<Keccak256, _, _>(
			&proof2.root,
			proof2.proof,
			data.len(),
			proof2.leaf_index,
			&proof2.leaf
		));

		// then
		assert_eq!(hex::encode(proof0.root), hex::encode(proof1.root));
		assert_eq!(hex::encode(proof2.root), hex::encode(proof1.root));

		assert!(!verify_proof::<Keccak256, _, _>(
			&hex!("fb3b3be94be9e983ba5e094c9c51a7d96a4fa2e5d8e891df00ca89ba05bb1239"),
			proof0.proof,
			data.len(),
			proof0.leaf_index,
			&proof0.leaf
		));

		assert!(!verify_proof::<Keccak256, _, _>(
			&proof0.root,
			vec![],
			data.len(),
			proof0.leaf_index,
			&proof0.leaf
		));
	}

	#[test]
	fn should_generate_and_verify_proof_complex() {
		// given
		let _ = env_logger::try_init();
		let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

		for l in 0..data.len() {
			// when
			let proof = merkle_proof::<Keccak256, _, _>(data.clone(), l);
			// then
			assert!(verify_proof::<Keccak256, _, _>(
				&proof.root,
				proof.proof,
				data.len(),
				proof.leaf_index,
				&proof.leaf
			));
		}
	}

	#[test]
	fn should_generate_and_verify_proof_large() {
		// given
		let _ = env_logger::try_init();
		let mut data = vec![];
		for i in 1..16 {
			for c in 'a'..'z' {
				if c as usize % i != 0 {
					data.push(c.to_string());
				}
			}

			for l in 0..data.len() {
				// when
				let proof = merkle_proof::<Keccak256, _, _>(data.clone(), l);
				// then
				assert!(verify_proof::<Keccak256, _, _>(
					&proof.root,
					proof.proof,
					data.len(),
					proof.leaf_index,
					&proof.leaf
				));
			}
		}
	}

	#[test]
	fn should_generate_and_verify_proof_large_tree() {
		// given
		let _ = env_logger::try_init();
		let mut data = vec![];
		for i in 0..6000 {
			data.push(format!("{}", i));
		}

		for l in (0..data.len()).step_by(13) {
			// when
			let proof = merkle_proof::<Keccak256, _, _>(data.clone(), l);
			// then
			assert!(verify_proof::<Keccak256, _, _>(
				&proof.root,
				proof.proof,
				data.len(),
				proof.leaf_index,
				&proof.leaf
			));
		}
	}

	#[test]
	#[should_panic]
	fn should_panic_on_invalid_leaf_index() {
		let _ = env_logger::try_init();
		merkle_proof::<Keccak256, _, _>(vec!["a"], 5);
	}
}
