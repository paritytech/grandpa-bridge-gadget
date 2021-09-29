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

use beefy_primitives::{
	crypto::{Public, Signature},
	MmrRootHash, ValidatorSet, ValidatorSetId,
};

mod error;
mod keyring;

pub use error::Error;
pub use keyring::Keyring;

/// Identifier for a finalized block at a specific height.
pub type BlockNumber = u64;

/// Commitment for a finalized block at [`BlockNumber`] containing a [`MmrRootHash`] as payload.
pub type Commitment = beefy_primitives::Commitment<BlockNumber, MmrRootHash>;

/// A [`Commitment`] containing a matching [`Signature`] from each validator of the current active [`ValidatorSet`].
pub type SignedCommitment = beefy_primitives::SignedCommitment<BlockNumber, MmrRootHash>;

pub struct LightClient {
	// BEEFY validator set
	validator_set: ValidatorSet<Public>,
}

impl LightClient {
	pub fn new() -> LightClient {
		LightClient {
			validator_set: ValidatorSet::empty(),
		}
	}

	pub fn import(&mut self, signed: SignedCommitment) -> Result<(), Error> {
		Err(Error::Commitment("not implemented yet".to_string()))
	}
}
