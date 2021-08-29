// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Utils for parsing user input

use sp_runtime::traits::Block as BlockT;
use std::{fmt::Debug, str::FromStr};

#[derive(Debug)]
pub(crate) enum BlockNumberOrHash {
	BlockNumber(u32),
	BlockHash(String),
}

pub(crate) fn block_number_or_hash(input: &str) -> Result<BlockNumberOrHash, String> {
	if let Ok(block_number) = u32::from_str(input) {
		Ok(BlockNumberOrHash::BlockNumber(block_number))
	} else {
		let (input, offset) = if input.starts_with("0x") {
			(&input[2..], 2)
		} else {
			(input, 0)
		};

		if let Some(pos) = input.chars().position(|c| !c.is_ascii_hexdigit()) {
			Err(format!(
				"Expected block hash, found illegal hex character at position: {}",
				offset + pos,
			))
		} else {
			Ok(BlockNumberOrHash::BlockHash(input.into()))
		}
	}
}

pub(crate) fn str_to_block_hash<Block>(block_hash: &str) -> sc_cli::Result<Block::Hash>
where
	Block: BlockT,
	<Block as BlockT>::Hash: FromStr,
	<<Block as BlockT>::Hash as FromStr>::Err: Debug,
{
	block_hash
		.parse::<<Block as BlockT>::Hash>()
		.map_err(|e| format!("Could not parse block hash: {:?}", e).into())
}

pub(crate) fn url(s: &str) -> Result<String, &'static str> {
	if s.starts_with("ws://") || s.starts_with("wss://") {
		// could use Url crate as well, but lets keep it simple for now.
		Ok(s.to_string())
	} else {
		Err("not a valid WS(S) url: must start with 'ws://' or 'wss://'")
	}
}
