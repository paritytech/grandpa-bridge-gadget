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

use beefy_primitives::ecdsa::AuthorityId;
use crate::cli::utils::{parse_hex, Authorities};
use parity_scale_codec::Decode;
use structopt::StructOpt;

/// Decode and uncompress encoded BEEFY id(s).
#[derive(StructOpt)]
#[structopt(about = "Decode and uncompress a vector of encoded BEEFY authority ids")]
pub struct UncompressAuthorities {
	/// A SCALE-encoded single BEEFY authority id (compressed public key).
	#[structopt(
		long,
		conflicts_with("authorities"),
		required_unless("authorities"),
		parse(try_from_str = beefy_id_from_hex),
	)]
	pub authority: Option<AuthorityId>,

	/// A SCALE-encoded vector of BEEFY authority ids (compressed public keys).
	///
	/// This can be obtained by querying `beefy.authorities`/`beefy.next_authorities` storage item
	/// of BEEFY pallet.
	#[structopt(
		long,
		conflicts_with("authority"),
		required_unless("authority"),
	)]
	pub authorities: Option<Authorities>,
}


impl UncompressAuthorities {
	pub fn run(self) -> anyhow::Result<()> {
		if let Some(id) = self.authority {
			return uncompress_beefy_ids(vec![id]);
		}

		if let Some(ids) = self.authorities {
			return uncompress_beefy_ids(ids.0);
		}

		anyhow::bail!("Neither argument given")
	}
}

fn uncompress_beefy_ids(ids: Vec<AuthorityId>) -> anyhow::Result<()> {
	for id in ids {
		let public = secp256k1::PublicKey::parse_slice(
			&*id.as_ref(),
			Some(secp256k1::PublicKeyFormat::Compressed),
		)?;
		println!("[{:?}] Uncompressed:\n\t {}", id, hex::encode(public.serialize()));
	}
	Ok(())
}

fn beefy_id_from_hex(id: &str) -> anyhow::Result<AuthorityId> {
	let encoded = parse_hex(id)?;
	let auth_id = AuthorityId::decode(&mut &*encoded)?;
	Ok(auth_id)
}
