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
use sp_keystore::SyncCryptoStore;

use beefy_primitives::KEY_TYPE;

use crate::error::{self};

pub(crate) trait BeefyKeystore<P>
where
	P: Pair,
{
	fn sign(&self, public: &P::Public, message: &[u8]) -> Result<P::Signature, error::Error>;

	fn verify(&self, public: &P::Public, sig: &P::Signature, message: &[u8]) -> bool;
}

impl BeefyKeystore<ecdsa::Pair> for dyn SyncCryptoStore {
	fn sign(&self, public: &ecdsa::Public, message: &[u8]) -> Result<ecdsa::Signature, error::Error> {
		let msg = keccak_256(message);

		let sig = SyncCryptoStore::ecdsa_sign_prehashed(&*self, KEY_TYPE, public, &msg)
			.map_err(|e| error::Error::Keystore(e.to_string()))?
			.ok_or_else(|| error::Error::Signature("ecdsa_sign_prehashed() failed".to_string()))?;

		Ok(sig)
	}

	fn verify(&self, public: &ecdsa::Public, sig: &ecdsa::Signature, message: &[u8]) -> bool {
		let msg = keccak_256(message);

		ecdsa::Pair::verify_prehashed(sig, &msg, public)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unit_cmp)]

	use super::BeefyKeystore;
	use beefy_primitives::KEY_TYPE;
	use sp_core::{ecdsa, keccak_256, Pair};
	use sp_keystore::{testing::KeyStore, SyncCryptoStore, SyncCryptoStorePtr};

	use crate::error::Error;

	#[test]
	fn sign_works() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let suri = "//Alice";
		let pair = ecdsa::Pair::from_string(suri, None).unwrap();

		let res = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, suri, pair.public().as_ref()).unwrap();
		assert_eq!((), res);

		let msg = b"are you involved or comitted?";
		let sig1 = store.sign(&pair.public(), msg).unwrap();

		let msg = keccak_256(b"are you involved or comitted?");
		let sig2 = SyncCryptoStore::ecdsa_sign_prehashed(&*store, KEY_TYPE, &pair.public(), &msg)
			.unwrap()
			.unwrap();

		assert_eq!(sig1, sig2);
	}

	#[test]
	fn sign_error() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let bob = ecdsa::Pair::from_string("//Bob", None).unwrap();
		let res = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, "//Bob", bob.public().as_ref()).unwrap();
		assert_eq!((), res);

		let alice = ecdsa::Pair::from_string("//Alice", None).unwrap();

		let msg = b"are you involved or comitted?";
		let sig = store.sign(&alice.public(), msg).err().unwrap();
		let err = Error::Signature("ecdsa_sign_prehashed() failed".to_string());
		assert_eq!(sig, err);
	}

	#[test]
	fn verify_works() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let suri = "//Alice";
		let pair = ecdsa::Pair::from_string(suri, None).unwrap();

		let res = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, suri, pair.public().as_ref()).unwrap();
		assert_eq!((), res);

		// `msg` and `sig` match
		let msg = b"are you involved or comitted?";
		let sig = store.sign(&pair.public(), msg).unwrap();
		assert!(store.verify(&pair.public(), &sig, msg));

		// `msg and `sig` don't match
		let msg = b"you are just involved";
		assert!(!store.verify(&pair.public(), &sig, msg));
	}
}
