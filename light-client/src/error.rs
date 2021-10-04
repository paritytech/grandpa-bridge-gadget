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

use core::{
	marker::{Send, Sync},
	write,
};

use crate::BlockNumber;

use displaydoc::Display;

#[derive(Display, Debug, PartialEq, Eq, Clone)]
pub enum Error {
	#[displaydoc("invalid validator set id: got {got:?} want {want:?}")]
	InvalidValidatorSet { got: u64, want: u64 },
	#[displaydoc("stale block number: got {got:?} best-known {best_known:?}")]
	StaleBlock { got: BlockNumber, best_known: BlockNumber },
	#[displaydoc("insufficient number of validator signatures: got {got:?} want {want:?}")]
	InsufficientSignatures { got: usize, want: usize },
	#[displaydoc("insufficient valid signatures: got {got:?} want {want:?}")]
	InsufficientValidSignatures { got: usize, want: usize },
	#[displaydoc("verify proof error: `{0}`")]
	Proof(String),
}

unsafe impl Send for Error {}

unsafe impl Sync for Error {}
