// Copyright 2019-2022 PureStake Inc.
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
// along with Moonbeam. If not, see <http://www.gnu.org/licenses/>.

pub mod chain;
pub mod cli;
pub mod client;
mod events;
mod runtime_upgrade;

use sc_cli::{CliConfiguration, SubstrateCli};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let cli = moonbeam_cli::Cli::from_args();
	let chain_id = cli.run.base.base.chain_id(false)?;

	match &*chain_id {
		id if id.starts_with("moonbase") => chain::moonbase::run()?,
		//"moonriver" => moonriver::run()?,
		//"moonbeam" => moonbeam::run()?,
		_ => panic!("Unsupported chain_id: {}", chain_id),
	};

	Ok(())
}
