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

mod merkle_tree;
mod mmr;
mod uncompress_authorities;
mod utils;

use structopt::StructOpt;

/// BEEFY utilities.
#[derive(StructOpt)]
#[structopt(about = "BEEFY utilities")]
pub enum Command {
	UncompressBeefyId(uncompress_authorities::UncompressAuthorities),
	BeefyIdMerkleTree(merkle_tree::BeefyMerkleTree),
	ParaHeadsMerkleTree(merkle_tree::ParaMerkleTree),
	Mmr(mmr::Mmr),
}

impl Command {
	/// Execute the command.
	pub fn run(self) -> anyhow::Result<()> {
		match self {
			Self::UncompressBeefyId(cmd) => cmd.run(),
			Self::BeefyIdMerkleTree(cmd) => cmd.run(),
			Self::ParaHeadsMerkleTree(cmd) => cmd.run(),
			Self::Mmr(cmd) => cmd.run(),
		}
	}
}

/// Parse relay CLI args.
pub fn parse_args() -> Command {
	Command::from_args()
}
