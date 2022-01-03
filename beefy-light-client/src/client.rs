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

use codec::Encode;
use beefy_primitives::{crypto::Public, ValidatorSet, ValidatorSetId};
use crate::{Commitment, Error, Keyring, SignedCommitment};

pub struct Client {
	/// active validator set
	active_set: ValidatorSet<Public>,
	/// next expected validator set id
	next_id: Option<ValidatorSetId>,
	/// latest valid commitment
	latest_commitment: Option<Commitment>,
}

impl Client {
	/// Return a [`Client`] using an intial validator set.
	pub fn new() -> Client {
		Client {
			active_set: ValidatorSet {
				validators: vec![Keyring::Alice.public()],
				id: 0,
			},
			next_id: None,
			latest_commitment: None,
		}
	}

	/// Verify and import a [`SignedCommitment`].
	pub fn import(&mut self, signed: SignedCommitment) -> Result<(), Error> {
		let commitment = self.verify_signed(signed)?;

		self.latest_commitment = Some(commitment);

		// silence clippy for now
		let _ = self.next_id;

		Ok(())
	}

	fn verify_signed(&self, signed: SignedCommitment) -> Result<Commitment, Error> {
		let SignedCommitment { commitment, signatures } = signed.clone();

		if self.active_set.id != commitment.validator_set_id {
			return Err(Error::InvalidValidatorSet {
				got: commitment.validator_set_id,
				want: self.active_set.id,
			});
		}

		let best_known = self.latest_commitment.as_ref().map(|c| c.block_number).unwrap_or(0);

		if commitment.block_number <= best_known {
			return Err(Error::StaleBlock {
				got: commitment.block_number,
				best_known,
			});
		}

		if signatures.len() != self.active_set.validators.len() {
			return Err(Error::InsufficientSignatures {
				got: signatures.len(),
				want: self.active_set.validators.len(),
			});
		}

		self.verify_signatures(signed)?;

		Ok(commitment)
	}

	fn verify_signatures(&self, signed: SignedCommitment) -> Result<(), Error> {
		if signed.no_of_signatures() < self.signature_threshold() {
			return Err(Error::InsufficientSignatures {
				got: signed.no_of_signatures(),
				want: self.signature_threshold(),
			});
		}

		let valid = signed
			.clone()
			.signatures
			.into_iter()
			.zip(self.active_set.validators.iter())
			.filter(|(sig, _)| sig.is_some())
			.map(|(sig, key)| Keyring::verify(key, &sig.unwrap(), &*signed.commitment.encode()))
			.filter(|b| *b)
			.count();

		if valid < self.signature_threshold() {
			return Err(Error::InsufficientValidSignatures {
				got: valid,
				want: self.signature_threshold(),
			});
		}

		Ok(())
	}

	fn signature_threshold(&self) -> usize {
		2 * self.active_set.validators.len() / 3 + 1
	}
}
