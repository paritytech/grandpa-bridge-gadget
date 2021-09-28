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

use std::{
	marker::{Send, Sync},
	write,
};

#[derive(PartialEq, Eq, Clone)]
pub enum Error {
	Commitment(String),
	Proof(String),
}

unsafe impl Send for Error {}

unsafe impl Sync for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Error::Commitment(msg) => write!(f, "validate commitment error: `{}`", msg)?,
			Error::Proof(msg) => write!(f, "verify proof error: `{}`", msg)?,
		}

		Ok(())
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Error::Commitment(msg) => write!(f, "validate commitment error: `{}`", msg)?,
			Error::Proof(msg) => write!(f, "verify proof error: `{}`", msg)?,
		}

		Ok(())
	}
}
