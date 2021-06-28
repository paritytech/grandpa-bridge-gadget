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
	let mut next = match merkelize_row::<H, _>(iter) {
		Ok(root) => return root,
		Err(next) if next.is_empty() => return Hash::default(),
		Err(next) => next,
	};

	loop {
		next = match merkelize_row::<H, _>(next.into_iter()) {
			Ok(root) => return root,
			Err(next) => next,
		};
	}
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
pub fn merkle_proof<H, I, T>(leaves: I, leaf_index: usize) -> (Hash, Vec<Hash>)
where
	H: Hasher,
	I: IntoIterator<Item = T>,
	T: AsRef<[u8]>,
{
	let is_even = leaf_index % 2 == 0;
	let mut proof = Vec::new();
	let iter = leaves.into_iter().enumerate().map(|(idx, l)| {
		let hash = H::hash(l.as_ref());
		// make sure to store hash of the leaf itself and adjacent node.
		if idx + 1 == leaf_index && !is_even {
			proof.push(hash);
		}
		if idx == leaf_index {
			proof.push(hash);
		}
		if idx == leaf_index + 1 && is_even {
			proof.push(hash);
		}
		hash
	});

	let result = merkelize_row::<H, _>(iter);
	assert!(!proof.is_empty(), "`leaf_index` is incorrect");

	let mut next = match result {
		Ok(root) => return (root, proof),
		Err(next) if next.is_empty() => return (Hash::default(), proof),
		Err(next) => next,
	};

	let mut index = leaf_index;
	loop {
		index = index / 2;
		index = if index % 2 == 0 { index + 1 } else { index - 1 };
		if next.len() > index {
			proof.push(next[index]);
		}
		next = match merkelize_row::<H, _>(next.into_iter()) {
			Ok(root) => return (root, proof),
			Err(next) => next,
		};
	}
}

/// Verify Merkle Proof correctness versus given root hash.
///
/// The proof is expected to contain leaf hash as the first element
/// and all adjacent nodes required to eventually by process of
/// concatenating and hashing end up with given root hash.
///
/// The proof must not contain the root hash.
pub fn verify_proof<H, P>(root: &Hash, proof: P) -> bool
where
	H: Hasher,
	P: IntoIterator<Item = Hash>,
{
	let mut combined = [0_u8; 64];
	let computed = proof.into_iter().reduce(|a, b| {
		if a < b {
			combined[0..32].copy_from_slice(&a);
			combined[32..64].copy_from_slice(&b);
		} else {
			combined[0..32].copy_from_slice(&b);
			combined[32..64].copy_from_slice(&a);
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
		hash
	});

	Some(root) == computed.as_ref()
}

/// Processes a single row (layer) of a tree by taking pairs of elements,
/// concatenating them, hashing and placing into resulting vector.
///
/// In case only one element is provided it is returned via `Ok` result, in any other case (also an
/// empty iterator) an `Err` with the inner nodes of upper layer is returned.
fn merkelize_row<H, I>(mut iter: I) -> Result<Hash, Vec<Hash>>
where
	H: Hasher,
	I: Iterator<Item = Hash>,
{
	#[cfg(feature = "debug")]
	log::debug!("[merkelize_row]");

	// TODO [ToDr] allocate externally.
	let mut next = Vec::with_capacity(iter.size_hint().0);
	let mut combined = [0_u8; 64];
	// TODO [ToDr] use chunks_exact?
	loop {
		let a = iter.next();
		let b = iter.next();
		#[cfg(feature = "debug")]
		log::debug!(
			"  {:?}\n  {:?}",
			a.as_ref().map(hex::encode),
			b.as_ref().map(hex::encode)
		);

		match (a, b) {
			(Some(a), Some(b)) => {
				if a < b {
					combined[0..32].copy_from_slice(&a);
					combined[32..64].copy_from_slice(&b);
				} else {
					combined[0..32].copy_from_slice(&b);
					combined[32..64].copy_from_slice(&a);
				}

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
			"5842148bc6ebeb52af882a317c765fccd3ae80589b21a9b8cbf21abb630e46a7",
			vec!["a", "b", "c"],
		);

		test(
			"7b84bec68b13c39798c6c50e9e40a0b268e3c1634db8f4cb97314eb243d4c514",
			vec!["a", "b", "a"],
		);

		test(
			"dc8e73fe6903148ff5079baecc043983625c23b39f31537e322cd0deee09fa9c",
			vec!["a", "b", "a", "b"],
		);

		test(
			"cc50382cfd3c9a617741e9a85efee8752b8feb95a2cbecd6365fb21366ce0c8c",
			vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
		);
	}

	#[test]
	fn should_generate_and_verify_proof() {
		// given
		let _ = env_logger::try_init();
		let data = vec!["a", "b", "c"];

		// when
		let (root0, proof0) = merkle_proof::<Keccak256, _, _>(data.clone(), 0);
		let (root1, proof1) = merkle_proof::<Keccak256, _, _>(data.clone(), 1);
		let (root2, proof2) = merkle_proof::<Keccak256, _, _>(data.clone(), 2);

		// then
		assert_eq!(hex::encode(root0), hex::encode(root1));
		assert_eq!(hex::encode(root2), hex::encode(root1));

		assert!(verify_proof::<Keccak256, _>(&root0, proof0.clone()));
		assert!(verify_proof::<Keccak256, _>(&root1, proof1));
		assert!(verify_proof::<Keccak256, _>(&root2, proof2));

		assert!(!verify_proof::<Keccak256, _>(
			&hex!("fb3b3be94be9e983ba5e094c9c51a7d96a4fa2e5d8e891df00ca89ba05bb1239"),
			proof0.clone()
		));

		assert!(!verify_proof::<Keccak256, _>(&root0, vec![]));
	}

	#[test]
	fn should_generate_and_verify_proof_complex() {
		// given
		let _ = env_logger::try_init();
		let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

		for _ in 0..data.len() {
			// when
			let (root, proof) = merkle_proof::<Keccak256, _, _>(data.clone(), 0);
			// then
			assert!(verify_proof::<Keccak256, _>(&root, proof));
		}
	}

	#[test]
	#[should_panic]
	fn should_panic_on_invalid_leaf_index() {
		let _ = env_logger::try_init();
		merkle_proof::<Keccak256, _, _>(vec!["a"], 5);
	}
}
