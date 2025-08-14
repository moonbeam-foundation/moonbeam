// Copyright 2019-2025 PureStake Inc.
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

use clap::Parser;
use moonbase_runtime::{
	MoonbasePrecompiles, PrecompileName as MoonbaseNames, Runtime as MoonbaseRuntime,
};
use moonbeam_runtime::{
	MoonbeamPrecompiles, PrecompileName as MoonbeamNames, Runtime as MoonbeamRuntime,
};
use moonriver_runtime::{
	MoonriverPrecompiles, PrecompileName as MoonriverNames, Runtime as MoonriverRuntime,
};
use precompile_utils::precompile_set::PrecompileKind;

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum, Default)]
enum Format {
	#[default]
	Debug,
	Json,
	JsonPretty,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Network {
	Moonbeam,
	Moonriver,
	Moonbase,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
	/// In which format the checks list should be exported.
	#[arg(short, long, value_enum)]
	format: Option<Format>,
	/// From which network we want to extract precompile data.
	#[arg(short, long, value_enum)]
	network: Network,
}

fn main() {
	let args = Args::parse();

	let mut summary = match args.network {
		Network::Moonbeam => MoonbeamPrecompiles::<MoonbeamRuntime>::new().summarize_checks(),
		Network::Moonbase => MoonbasePrecompiles::<MoonbaseRuntime>::new().summarize_checks(),
		Network::Moonriver => MoonriverPrecompiles::<MoonriverRuntime>::new().summarize_checks(),
	};

	for item in summary.iter_mut() {
		let name = match (&args.network, &item.precompile_kind) {
			(Network::Moonbeam, PrecompileKind::Single(address)) => {
				MoonbeamNames::from_address(*address).map(|v| format!("{v:?}"))
			}
			(Network::Moonbase, PrecompileKind::Single(address)) => {
				MoonbaseNames::from_address(*address).map(|v| format!("{v:?}"))
			}
			(Network::Moonriver, PrecompileKind::Single(address)) => {
				MoonriverNames::from_address(*address).map(|v| format!("{v:?}"))
			}
			(_, PrecompileKind::Prefixed(prefix)) if prefix == &[0xff, 0xff, 0xff, 0xff] => {
				Some("ForeignAssets".into())
			}
			(_, PrecompileKind::Prefixed(prefix)) if prefix == &[0xff, 0xff, 0xff, 0xfe] => {
				Some("LocalAssets".into())
			}
			_ => None,
		};

		item.name = name;
	}

	let output = match args.format.unwrap_or_default() {
		Format::Debug => format!("{summary:#?}"),
		Format::Json => serde_json::to_string(&summary).expect("to serialize correctly"),
		Format::JsonPretty => {
			serde_json::to_string_pretty(&summary).expect("to serialize correctly")
		}
	};

	println!("{output}");
}
