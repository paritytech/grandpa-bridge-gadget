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

use beefy_primitives::crypto::AuthorityId;
use parity_scale_codec::Decode;

/// Parse hex string to a vector of bytes.
pub fn parse_hex(hex: &str) -> anyhow::Result<Vec<u8>> {
	let s = if hex.starts_with("0x") {
		&hex.as_bytes()[2..]
	} else {
		&hex.as_bytes()
	};

	Ok(hex::decode(s)?)
}

/// A wrapper struct to overcome structopt's `Vec` special handling.
pub struct Bytes(pub Vec<u8>);
impl std::str::FromStr for Bytes {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> anyhow::Result<Self> {
		parse_hex(s).map(Bytes)
	}
}

/// Overcome issue with structopt's `Vec` handling.
pub struct Authorities(pub Vec<AuthorityId>);
impl std::str::FromStr for Authorities {
	type Err = anyhow::Error;

	fn from_str(id: &str) -> anyhow::Result<Self> {
		let encoded = parse_hex(id)?;
		let auth_ids = Vec::<AuthorityId>::decode(&mut &*encoded)?;
		Ok(Self(auth_ids))
	}
}
