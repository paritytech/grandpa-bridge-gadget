// Copyright (C) 2020 Parity Technologies (UK) Ltd.
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

mod light_client;

use self::light_client::{validator_set, Commitment, Error, Payload, SignedCommitment};

#[test]
fn light_client_should_make_progress() {
	let mut lc = light_client::new();

	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 1,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	assert!(result.is_ok());
	assert_eq!(lc.last_payload(), &Payload::new(1));
}

#[test]
fn light_client_should_reject_invalid_validator_set() {
	let mut lc = light_client::new();

	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 1,
			validator_set_id: 1,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	assert_eq!(result, Err(Error::InvalidValidatorSetId { expected: 0, got: 1 }));
	assert_eq!(lc.last_commitment(), None);
}
