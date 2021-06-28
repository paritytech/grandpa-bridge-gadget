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

#[cfg(not(feature = "std"))]
use core::vec::Vec;

pub type Output = [u8; 32];
pub trait Hasher {
	fn hash(data: &[u8]) -> Output;
}

pub fn merkle_root<H, I, T>(leaves: I) -> Output
where
	H: Hasher,
	I: IntoIterator<Item = T> + Clone,
	T: AsRef<[u8]>,
{
	#[cfg(feature = "debug")]
	println!(
		"Leaves: {:?}",
		leaves
			.clone()
			.into_iter()
			.map(|l| H::hash(l.as_ref()))
			.map(|x| hex::encode(&x))
			.collect::<Vec<_>>()
	);
	let mut iter = leaves.into_iter().map(|l| H::hash(l.as_ref()));
	let mut next = match merkelize_row::<H, _>(iter) {
		Ok(root) => return root,
		Err(next) if next.is_empty() => return Output::default(),
		Err(next) => next,
	};

	loop {
		#[cfg(feature = "debug")]
		println!("Layer: {:?}", next.iter().map(|x| hex::encode(x)).collect::<Vec<_>>());
		next = match merkelize_row::<H, _>(next.into_iter()) {
			Ok(root) => return root,
			Err(next) => next,
		};
	}
}

fn merkelize_row<H, I>(mut iter: I) -> Result<Output, Vec<Output>>
where
	H: Hasher,
	I: Iterator<Item = Output>,
{
	// TODO [ToDr] allocate externally.
	let mut next = Vec::with_capacity(iter.size_hint().0);
	let mut combined = [0_u8; 64];
	loop {
		let a = iter.next();
		let b = iter.next();

		match (a, b) {
			(Some(a), Some(b)) => {
				combined[0..32].copy_from_slice(&a);
				combined[32..64].copy_from_slice(&b);
				// TODO sort?

				next.push(H::hash(&combined));
			}
			// Odd number of items. Promote the item to the upper layer.
			(Some(a), None) if !next.is_empty() => {
				next.push(a);
				// combined[0..32].copy_from_slice(&a);
				// combined[32..64].copy_from_slice(&a);
			}
			// Last item = root.
			(Some(a), None) => {
				return Ok(a);
			}
			// Finish up, no more items.
			_ => {
				return Err(next);
			}
		}
	}
}

pub fn merkle_proof<H, I, T>(leaves: I) -> ()
// impl Iterator<Item = Box<[u8]>>
where
	H: Hasher,
	I: IntoIterator<Item = T>,
	T: AsRef<[u8]>,
{
	unimplemented!()
}

pub fn verify_proof<H, P, T>(root: &Output, proof: P) -> bool
where
	H: Hasher,
	P: IntoIterator<Item = T>,
	T: AsRef<[u8]>,
{
	unimplemented!()
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;
	use tiny_keccak::{Hasher as _, Keccak};

	struct Keccak256;
	impl Hasher for Keccak256 {
		fn hash(data: &[u8]) -> Output {
			let mut keccak = Keccak::v256();
			keccak.update(data);
			let mut output = [0_u8; 32];
			keccak.finalize(&mut output);
			output
		}
	}

	#[test]
	fn should_generate_empty_root() {
		// given
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
}
