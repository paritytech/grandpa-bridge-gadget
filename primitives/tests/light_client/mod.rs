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

use beefy_primitives::{self as bp, ValidatorSetId};

pub mod merkle_tree;
pub mod validator_set;

/// A marker struct for validator set merkle tree.
#[derive(Debug)]
pub struct ValidatorSetTree;

/// A marker struct for the MMR.
#[derive(Debug)]
pub struct Mmr;

#[derive(Debug, PartialEq, Eq)]
pub struct Payload {
	pub next_validator_set: Option<merkle_tree::Root<ValidatorSetTree>>,
	pub mmr: merkle_tree::Root<Mmr>,
}

impl Payload {
	pub fn new(root: u32) -> Self {
		Self {
			next_validator_set: None,
			mmr: root.into(),
		}
	}
}

pub type BlockNumber = u64;
pub type Commitment = bp::Commitment<BlockNumber, Payload>;
pub type SignedCommitment = bp::SignedCommitment<BlockNumber, Payload, validator_set::Signature>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	InvalidValidatorSetId {
		expected: ValidatorSetId,
		got: ValidatorSetId,
	}
}

pub struct LightClient {
	validator_set: (ValidatorSetId, Vec<validator_set::Public>),
	last_commitment: Option<Commitment>,
}

impl LightClient {
	pub fn import(
		&mut self,
		commitment: SignedCommitment,
	) -> Result<(), Error> {
		// TODO proper verification
		// 1. validator_set
		// 2. block numbers
		// 3. Is epoch change
		// 4. number of signatures
		// 5. signatures validity
		self.last_commitment = Some(commitment.commitment);
		Ok(())
	}

	pub fn import_epoch(
		&mut self,
		commitment: SignedCommitment,
		validator_set_proof: merkle_tree::Proof<ValidatorSetTree, Vec<validator_set::Public>>,
	) -> Result<(), Error> {
		todo!()
	}

	pub fn last_commitment(&self) -> Option<&Commitment> {
		self.last_commitment.as_ref()
	}

	pub fn last_payload(&self) -> &Payload {
		&self.last_commitment().unwrap().payload
	}
}

pub fn new() -> LightClient {
	LightClient {
		validator_set: (0, vec![validator_set::Public(0)]),
		last_commitment: None,
	}
}
