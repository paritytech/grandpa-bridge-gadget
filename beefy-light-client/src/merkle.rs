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

use arber::{self, Error, MerkleMountainRange, VecStore};
use codec::Encode;

use crate::{Commitment, Keyring};

#[allow(clippy::upper_case_acronyms)]
type MMR = MerkleMountainRange<arber::Hash, VecStore<arber::Hash>>;

fn signature_mmr(commitment: &Commitment, validators: &[Keyring]) -> Result<MMR, Error> {
	let mut mmr = MMR::new(VecStore::new());

	for v in validators {
		let sig = v.sign(&*commitment.encode());
		let hash = arber::Hash::from_vec(sig.as_ref());
		mmr.append(&hash)?;
	}

	Ok(mmr)
}

#[cfg(test)]
mod tests {
	use sp_core::H256;

	use super::*;

	#[test]
	fn signature_mmr_works() -> Result<(), Error> {
		let commitment = Commitment {
			payload: H256::from_low_u64_le(42),
			block_number: 2,
			validator_set_id: 0,
		};

		let validators = vec![Keyring::Alice, Keyring::Bob, Keyring::Charlie];

		let mmr = signature_mmr(&commitment, &validators)?;

		assert_eq!(4, mmr.size);

		Ok(())
	}
}
