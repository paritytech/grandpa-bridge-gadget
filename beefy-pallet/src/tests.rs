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

#![cfg(test)]

use crate::mock::{mock_beefy_id, new_test_ext, Beefy};

#[test]
fn genesis_session_initializes_authorities() {
	let want = vec![mock_beefy_id(1), mock_beefy_id(2), mock_beefy_id(3)];

	new_test_ext(vec![1, 2, 3]).execute_with(|| {
		let authorities = Beefy::authorities();

		assert!(authorities.len() == 3);
		assert_eq!(want[0], authorities[0]);
		assert_eq!(want[1], authorities[1]);
		assert_eq!(want[2], authorities[2]);

		assert!(Beefy::validator_set_id() == 0);

		let next_authorities = Beefy::next_authorities();

		assert!(next_authorities.len() == 3);
		assert_eq!(want[0], next_authorities[0]);
		assert_eq!(want[1], next_authorities[1]);
		assert_eq!(want[2], next_authorities[2]);
	});
}
