// Copyright (C) 2020 - 2021 Parity Technologies (UK) Ltd.
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

use std::vec;

use beefy_primitives::{
	mmr::{BeefyNextAuthoritySet, MmrLeafVersion},
	ValidatorSet,
};
use codec::{Decode, Encode};
use hex_literal::hex;

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{traits::Keccak256, DigestItem};

use frame_support::traits::OnInitialize;

use crate::mock::*;

fn init_block(block: u64) {
	System::set_block_number(block);
	Session::on_initialize(block);
	Mmr::on_initialize(block);
}

pub fn beefy_log(log: ConsensusLog<BeefyId>) -> DigestItem<H256> {
	DigestItem::Consensus(BEEFY_ENGINE_ID, log.encode())
}

fn offchain_key(pos: usize) -> Vec<u8> {
	(<Test as pallet_mmr::Config>::INDEXING_PREFIX, pos as u64).encode()
}

fn read_mmr_leaf(ext: &mut TestExternalities, index: usize) -> MmrLeaf {
	type Node = pallet_mmr_primitives::DataOrHash<Keccak256, MmrLeaf>;
	ext.persist_offchain_overlay();
	let offchain_db = ext.offchain_db();
	offchain_db
		.get(&offchain_key(index))
		.map(|d| Node::decode(&mut &*d).unwrap())
		.map(|n| match n {
			Node::Data(d) => d,
			_ => panic!("Unexpected MMR node."),
		})
		.unwrap()
}

#[test]
fn should_contain_mmr_digest() {
	let mut ext = new_test_ext(vec![1, 2, 3, 4]);
	ext.execute_with(|| {
		init_block(1);

		assert_eq!(
			System::digest().logs,
			vec![beefy_log(ConsensusLog::MmrRoot(
				hex!("108e5ad4890955bc296b0ac4ef62c8ee251eade7c345073732a26bbac6ae80aa").into()
			))]
		);

		// unique every time
		init_block(2);

		assert_eq!(
			System::digest().logs,
			vec![
				beefy_log(ConsensusLog::MmrRoot(
					hex!("108e5ad4890955bc296b0ac4ef62c8ee251eade7c345073732a26bbac6ae80aa").into()
				)),
				beefy_log(ConsensusLog::AuthoritiesChange(ValidatorSet {
					validators: vec![mock_beefy_id(3), mock_beefy_id(4),],
					id: 1,
				})),
				beefy_log(ConsensusLog::MmrRoot(
					hex!("e3d39d6b720e4a1694e73e4845c11fd420291894e8384173d35329552d74aeb5").into()
				)),
			]
		);
	});
}

#[test]
fn should_contain_valid_leaf_data() {
	let mut ext = new_test_ext(vec![1, 2, 3, 4]);
	ext.execute_with(|| {
		init_block(1);
	});

	let mmr_leaf = read_mmr_leaf(&mut ext, 0);

	assert_eq!(
		mmr_leaf,
		MmrLeaf {
			version: MmrLeafVersion::new(1, 5),
			parent_number_and_hash: (0_u64, H256::repeat_byte(0x45)),
			beefy_next_authority_set: BeefyNextAuthoritySet {
				id: 1,
				len: 2,
				root: hex!("dacdb4dddef8f3bbfc4cbc893f670fe368c76179b05f7a406fd8cf7e35fec482").into(),
			},
			parachain_heads: hex!("18128e4279e142bf5a42dae8b53a66c4ab0d63a1a61d5270370d678fa92cc999").into(),
			extended_data: (),
		}
	);
}
