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

#![warn(missing_docs)]

use sp_core::{ecdsa, keccak_256, Pair};

use beefy_primitives::crypto;

/// Set of test accounts using [`beefy_primitives::crypto`] types.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumIter)]
pub enum Keyring {
	Alice,
	Bob,
	Charlie,
	Dave,
	Eve,
	Ferdie,
	One,
	Two,
}

impl Keyring {
	/// Sign `msg`.
	pub fn sign(self, msg: &[u8]) -> crypto::Signature {
		let msg = keccak_256(msg);
		ecdsa::Pair::from(self).sign_prehashed(&msg).into()
	}

	/// Return key pair.
	pub fn pair(self) -> crypto::Pair {
		ecdsa::Pair::from_string(self.to_seed().as_str(), None).unwrap().into()
	}

	/// Return public key.
	pub fn public(self) -> crypto::Public {
		self.pair().public()
	}

	/// Return seed string.
	pub fn to_seed(self) -> String {
		format!("//{}", self)
	}

	/// Iterator over all test accounts
	pub fn iter() -> impl (Iterator<Item = Keyring>) {
		<Self as strum::IntoEnumIterator>::iter()
	}
}

impl From<Keyring> for crypto::Pair {
	fn from(k: Keyring) -> Self {
		k.pair()
	}
}

impl From<Keyring> for ecdsa::Pair {
	fn from(k: Keyring) -> Self {
		k.pair().into()
	}
}

#[cfg(test)]
mod tests {
	use super::Keyring;
	use beefy_primitives::crypto;
	use sp_core::{ecdsa, keccak_256, Pair};

	#[test]
	fn verify_should_work() {
		let msg = keccak_256(b"I am Alice!");
		let sig = Keyring::Alice.sign(b"I am Alice!");

		assert!(ecdsa::Pair::verify_prehashed(
			&sig.clone().into(),
			&msg,
			&Keyring::Alice.public().into(),
		));

		// different public key -> fail
		assert!(!ecdsa::Pair::verify_prehashed(
			&sig.clone().into(),
			&msg,
			&Keyring::Bob.public().into(),
		));

		let msg = keccak_256(b"I am not Alice!");

		// different msg -> fail
		assert!(!ecdsa::Pair::verify_prehashed(
			&sig.into(),
			&msg,
			&Keyring::Alice.public().into(),
		));
	}

	#[test]
	fn pair_works() {
		let want = crypto::Pair::from_string("//Alice", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Alice.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Bob", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Bob.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Charlie", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Charlie.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Dave", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Dave.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Eve", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Eve.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Ferdie", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Ferdie.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//One", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::One.pair().to_raw_vec();
		assert_eq!(want, got);

		let want = crypto::Pair::from_string("//Two", None)
			.expect("Pair failed")
			.to_raw_vec();
		let got = Keyring::Two.pair().to_raw_vec();
		assert_eq!(want, got);
	}
}
