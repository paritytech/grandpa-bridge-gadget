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

use sp_core::{
	ecdsa::{Pair, Public, Signature},
	Pair as PairT,
};

/// Set of test accounts using ECDSA crypto
///
/// The reason why we do not add `Keyring` to Substrats's `sp_keyring` right
/// away is because `sp_keyring` is based on the assumption, that a public key is
/// 32 bytes long. This is not true for compressed ECDSA public keys which are 33
/// bytes long. This causes all kinds of incompatibilities between the existing
/// Substrate keyrings and an ECDSA based keyring.
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
	pub fn sign(self, msg: &[u8]) -> Signature {
		Pair::from(self).sign(msg)
	}

	pub fn pair(self) -> Pair {
		Pair::from_string(&format!("//{}", <&'static str>::from(self)), None)
			.expect("static values are known good; qed")
	}

	pub fn public(self) -> Public {
		self.pair().public()
	}

	pub fn to_seed(self) -> String {
		format!("//{}", self)
	}

	/// Iterator over all test accounts
	pub fn iter() -> impl (Iterator<Item = Keyring>) {
		<Self as strum::IntoEnumIterator>::iter()
	}
}

impl From<Keyring> for &'static str {
	fn from(k: Keyring) -> Self {
		match k {
			Keyring::Alice => "Alice",
			Keyring::Bob => "Bob",
			Keyring::Charlie => "Charlie",
			Keyring::Dave => "Dave",
			Keyring::Eve => "Eve",
			Keyring::Ferdie => "Ferdie",
			Keyring::One => "One",
			Keyring::Two => "Two",
		}
	}
}

impl From<Keyring> for Pair {
	fn from(k: Keyring) -> Self {
		k.pair()
	}
}

#[cfg(test)]
mod tests {
	use super::Keyring;
	use sp_core::{ecdsa::Pair, Pair as PairT};

	#[test]
	fn verify_should_work() {
		assert!(Pair::verify(
			&Keyring::Alice.sign(b"I am Alice!"),
			b"I am Alice!",
			&Keyring::Alice.public(),
		));

		assert!(!Pair::verify(
			&Keyring::Alice.sign(b"I am Alice!"),
			b"I am Bob!",
			&Keyring::Alice.public(),
		));

		assert!(!Pair::verify(
			&Keyring::Alice.sign(b"I am Alice!"),
			b"I am Alice!",
			&Keyring::Bob.public(),
		));
	}
}
