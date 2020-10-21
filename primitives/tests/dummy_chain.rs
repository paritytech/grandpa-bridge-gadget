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
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 2,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	// then
	assert!(result.is_ok());
	assert_eq!(lc.last_payload(), &Payload::new(1));
}

#[test]
fn light_client_should_reject_invalid_validator_set() {
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 1,
			validator_set_id: 1,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	// then
	assert_eq!(result, Err(Error::InvalidValidatorSetId { expected: 0, got: 1 }));
	assert_eq!(lc.last_commitment(), None);
}

#[test]
fn light_client_should_reject_set_transitions_without_validator_proof() {
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 1,
			validator_set_id: 0,
			is_set_transition_block: true,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	// then
	assert_eq!(result, Err(Error::InvalidValidatorSetProof));
	assert_eq!(lc.last_commitment(), None);
}

#[test]
fn light_client_should_reject_older_block() {
	// given
	let mut lc = light_client::new();
	// jump to 10
	lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 10,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	}).unwrap();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(0.into()))],
	});

	// then
	assert_eq!(result, Err(Error::OldBlock { best_known: 10, got: 5 }));
}

#[test]
fn light_client_should_reject_if_not_enough_signatures() {
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![None],
	});

	// then
	assert_eq!(result, Err(Error::NotEnoughValidSignatures {
		expected: 1,
		got: 0,
		valid: None,
	}));
}

#[test]
fn light_client_should_reject_if_too_many_or_too_little_signatures() {
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![None, None],
	});
	let result2 = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![],
	});

	// then
	assert_eq!(result, Err(Error::InvalidNumberOfSignatures {
		expected: 1,
		got: 2,
	}));
	assert_eq!(result2, Err(Error::InvalidNumberOfSignatures {
		expected: 1,
		got: 0,
	}));
}

#[test]
fn light_client_should_reject_if_not_enough_valid_signatures() {
	// given
	let mut lc = light_client::new();

	// when
	let result = lc.import(SignedCommitment {
		commitment: Commitment {
			payload: Payload::new(1),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		},
		signatures: vec![Some(validator_set::Signature::ValidFor(1.into()))],
	});

	// then
	assert_eq!(result, Err(Error::NotEnoughValidSignatures {
		expected: 1,
		got: 1,
		valid: Some(0),
	}));
}
