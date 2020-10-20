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

/// A commitment signed by Grandpa validators as part of BEEFY protocol.
///
/// The commitment contins a [payload] extracted from the finalized block at height [block_number].
/// Grandpa validators collect signatures on commitments and a stream of such signed commitments
/// (see [SignedCommitment]) forms the BEEFY protocol.
#[derive(Debug, PartialEq, Eq, codec::Encode, codec::Decode)]
pub struct Commitment<TBlockNumber, TPayload> {
    /// The payload being signed.
    ///
    /// This should be some form of cummulative representation of the chain (think MMR root hash).
    /// For transition blocks it also MUST contain details of the next validator set.
    pub payload: TPayload,

    /// Finalized block number this commitment is for.
    ///
    /// Grandpa validators agree on a block they create a commitment for and start collecting
	/// signature. This process is called a round.
	/// There might be multiple rounds in progress (depending on the block choice rule), however
	/// since the payload is supposed to be cummulative, it is not required to import all
	/// commitments.
	/// BEEFY light client is expected to import at least one commitment per epoch (the one with
	/// [is_set_transition_block] set), but is free to import as many as it requires.
    pub block_number: TBlockNumber,

    /// BEEFY valitor set supposed to sign this comitment.
	///
	/// Validator set is changing once per epoch in the commitment with [is_set_transition_block]
	/// set to `true`. Such "epoch commitments" MUST provide the light client with details of the
	/// new validator set as part of the payload. The protocol itself doesn't enforce how these
	/// details are provided though.
    pub validator_set_id: u64,

    /// Indicator of the last block of the epoch.
    ///
    /// The payload will contain some form of the NEW validator set public keys information,
	/// yet the block is signed by the current validator set.
    /// When this commitment is imported, the client MUST increment the `validator_set_id`.
    pub is_set_transition_block: bool,
}

// impl<TBlockNumber, TPayload> core::cmp::Ord for Commitment<

/// A commitment with matching Grandpa validators' signatures.
#[derive(Debug, PartialEq, Eq, codec::Encode, codec::Decode)]
pub struct SignedCommitment<TBlockNumber, TPayload, TSignature> {
    /// The commitment signatures are collected for.
    pub commitment: Commitment<TBlockNumber, TPayload>,
    /// Grandpa validators' signatures for the commitment.
	///
	/// The length of this `Vec` must match number of validators in the current set (see
	/// [Commitment::validator_set_id]).
    pub signatures: Vec<Option<TSignature>>,
}

impl<TBlockNumber, TPayload, TSignature> SignedCommitment<TBlockNumber, TPayload, TSignature> {
	/// Return the number of collected signatures.
	pub fn no_of_signatures(&self) -> usize {
		self.signatures.iter().filter(|x| x.is_some()).count()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	type TestCommitment = Commitment<u128, String>;
	type TestSignedCommitment = SignedCommitment<u128, String, Vec<u8>>;

	#[test]
	fn commitment_encode_decode() {
		// given
		let commitment: TestCommitment = Commitment {
			payload: "Hello World!".into(),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		};

		// when
		let encoded = codec::Encode::encode(&commitment);
		let decoded = TestCommitment::decode(&mut &*encoded);

		// then
		assert_eq!(decoded, commitment);
		assert_eq!(encoded, hex_literal::hex!("ff"));
	}

	#[test]
	fn signed_commitment_encode_decode() {
		// given
		let commitment: TestCommitment = Commitment {
			payload: "Hello World!".into(),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		};
		let signed = SignedCommitment {
			commitment,
			signatures: vec![
				None,
				None,
				Some(vec![1, 2, 3, 4]),
				Some(vec![5, 6, 7, 8]),
			],
		};

		// when
		let encoded = codec::Encode::encode(&signed);
		let decoded = TestSignedCommitment::decode(&mut &*encoded);

		// then
		assert_eq!(decoded, signed);
		assert_eq!(encoded, hex_literal::hex!("ff"));
	}

	#[test]
	fn signed_commitment_count_signatures() {
		// given
		let commitment: TestCommitment = Commitment {
			payload: "Hello World!".into(),
			block_number: 5,
			validator_set_id: 0,
			is_set_transition_block: false,
		};
		let mut signed = SignedCommitment {
			commitment,
			signatures: vec![
				None,
				None,
				Some(vec![1, 2, 3, 4]),
				Some(vec![5, 6, 7, 8]),
			],
		};
		assert_eq!(signed.no_of_signatures(), 2);

		// when
		signed.signatures[2] = None;

		// then
		assert_eq!(signed.no_of_signatures(), 1);
	}

	#[test]
	fn commitment_ordering() {
		assert_eq!(true, false);
	}
}
