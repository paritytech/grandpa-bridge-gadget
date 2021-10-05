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

use sp_core::H256;

use codec::Encode;

use light_client::{Client, Commitment, Keyring, SignedCommitment};

#[test]
fn import_with_initial_validator_set() {
	let mut client = Client::new();

	let commitment = Commitment {
		payload: (H256::from_low_u64_le(42)),
		block_number: 2,
		validator_set_id: 0,
	};

	let sig = Keyring::Alice.sign(&*commitment.encode());

	let signed = SignedCommitment {
		commitment,
		signatures: vec![Some(sig)],
	};

	let res = client.import(signed);
	assert_eq!(res, Ok(()))
}
