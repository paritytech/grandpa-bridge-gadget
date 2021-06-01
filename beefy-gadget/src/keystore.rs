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

use sp_core::{ecdsa, keccak_256, Pair};
use sp_keystore::{Error, SyncCryptoStore};

use beefy_primitives::KEY_TYPE;

pub(crate) trait BeefyKeystore<P>
where
	P: Pair,
{
	fn sign(&self, public: P::Public, message: &[u8]) -> Result<P::Signature, Error>;
}

impl BeefyKeystore<ecdsa::Pair> for dyn SyncCryptoStore {
	fn sign(&self, public: ecdsa::Public, message: &[u8]) -> Result<ecdsa::Signature, Error> {
		let msg = keccak_256(message);
		let sig = SyncCryptoStore::ecdsa_sign_prehashed(&*self, KEY_TYPE, &public, &msg)?;

		Ok(sig.unwrap())
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unit_cmp)]

	use super::BeefyKeystore;
	use beefy_primitives::KEY_TYPE;
	use sp_core::{ecdsa, keccak_256, Pair};
	use sp_keystore::{testing::KeyStore, SyncCryptoStore, SyncCryptoStorePtr};

	#[test]
	fn beefy_keystore_sign_works() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let suri = "//Alice";
		let pair = ecdsa::Pair::from_string(suri, None).unwrap();

		let res = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, suri, pair.public().as_ref()).unwrap();
		assert_eq!((), res);

		let msg = b"this should be a hashed message";
		let sig1 = store.sign(pair.public(), msg).unwrap();

		let msg = keccak_256(b"this should be a hashed message");
		let sig2 = SyncCryptoStore::ecdsa_sign_prehashed(&*store, KEY_TYPE, &pair.public(), &msg)
			.unwrap()
			.unwrap();

		assert_eq!(sig1, sig2);
	}
}
