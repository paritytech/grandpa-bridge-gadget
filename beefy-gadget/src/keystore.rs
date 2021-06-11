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

use std::convert::TryInto;

use sp_application_crypto::Public;
use sp_core::{keccak_256, Pair};
use sp_keystore::SyncCryptoStore;

use beefy_primitives::{ecdsa, KEY_TYPE};

use crate::error;

/// BEEFY specifc keystore which allows to customize message signature related
/// crypto functions based on a concrete type for [`sp_core::Pair`]. This conrete
/// type is also known as the `BEEFY Key`.
pub trait BeefyKeystore<P>: Sync + Send + 'static
where
	P: Pair,
{
	/// Check if the keystore contains a private key for one of the public keys
	/// contained in `keys`. A public key with a matching private key is known
	/// as a local authority id.
	///
	/// Return the public key for which we also do have a private key. If no
	/// matching private key is found, `None` will be returned.
	fn local_id(&self, keys: &[P::Public]) -> Option<P::Public>;

	/// Sign `message` with the `public` key.
	///
	/// Note that `message` usually will be pre-hashed before being singed.
	///
	/// Return the message signature or an error in case of failure.
	fn sign(&self, public: &P::Public, message: &[u8]) -> Result<P::Signature, error::Error>;

	/// Use the `public` key to verify that `sig` is a valid signature for `message`.
	///
	/// Return `true` if the signature is authentic, `false` otherwise.
	fn verify(&self, public: &P::Public, sig: &P::Signature, message: &[u8]) -> bool;
}

impl BeefyKeystore<ecdsa::Pair> for std::sync::Arc<dyn SyncCryptoStore> {
	fn local_id(&self, keys: &[ecdsa::Public]) -> Option<ecdsa::Public> {
		for key in keys {
			if SyncCryptoStore::has_keys(&**self, &[(key.to_raw_vec(), KEY_TYPE)]) {
				return Some(key.clone());
			}
		}

		None
	}

	fn sign(&self, public: &ecdsa::Public, message: &[u8]) -> Result<ecdsa::Signature, error::Error> {
		let msg = keccak_256(message);
		let public = public.as_ref();

		let sig = SyncCryptoStore::ecdsa_sign_prehashed(&**self, KEY_TYPE, public, &msg)
			.map_err(|e| error::Error::Keystore(e.to_string()))?
			.ok_or_else(|| error::Error::Signature("ecdsa_sign_prehashed() failed".to_string()))?;

		// check that `sig` has the expected result type
		let sig = sig
			.clone()
			.try_into()
			.map_err(|_| error::Error::Signature(format!("invalid signature {:?} for key {:?}", sig, public)))?;

		Ok(sig)
	}

	fn verify(&self, public: &ecdsa::Public, sig: &ecdsa::Signature, message: &[u8]) -> bool {
		let msg = keccak_256(message);
		let sig = sig.as_ref();
		let public = public.as_ref();

		sp_core::ecdsa::Pair::verify_prehashed(sig, &msg, public)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unit_cmp)]

	use super::BeefyKeystore;
	use beefy_primitives::{ecdsa, KEY_TYPE};
	use sp_core::{keccak_256, Pair};
	use sp_keystore::{testing::KeyStore, SyncCryptoStore, SyncCryptoStorePtr};

	use crate::error::Error;

	#[test]
	fn local_id_works() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let alice = ecdsa::Pair::from_string("//Alice", None).unwrap();
		let _ = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, "//Alice", alice.public().as_ref()).unwrap();

		let bob = ecdsa::Pair::from_string("//Bob", None).unwrap();
		let charlie = ecdsa::Pair::from_string("//Charlie", None).unwrap();

		let mut keys = vec![bob.public(), charlie.public()];

		let local_id = store.local_id(&keys);
		assert!(local_id.is_none());

		keys.push(alice.public());

		let local_id = store.local_id(&keys).unwrap();
		assert_eq!(local_id, alice.public());
	}

	#[test]
	fn sign_works() {
		let store: SyncCryptoStorePtr = KeyStore::new().into();

		let suri = "//Alice";
		let pair = sp_core::ecdsa::Pair::from_string(suri, None).unwrap();

		let res = SyncCryptoStore::insert_unknown(&*store, KEY_TYPE, suri, pair.public().as_ref()).unwrap();
		assert_eq!((), res);

		let msg = b"are you involved or comitted?";
		let sig1 = store.sign(&pair.public().into(), msg).unwrap();

		let msg = keccak_256(b"are you involved or comitted?");
		let sig2 = SyncCryptoStore::ecdsa_sign_prehashed(&*store, KEY_TYPE, &pair.public(), &msg)
			.unwrap()
			.unwrap();

		assert_eq!(sig1, sig2.into());
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
