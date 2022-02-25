// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod command;
mod tests;
mod txn_signer;

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct PerfCmd {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: sc_cli::SharedParams,

	#[clap(
		long = "working-dir",
		help = "Used for temp blockchain data. Should exist on desired test hardware.",
		required = true
	)]
	pub working_dir: PathBuf,

	#[clap(
		long = "output-file",
		help = "File where results should be printed (STDOUT if omitted)."
	)]
	pub output_file: Option<PathBuf>,

	#[clap(long, value_name = "CHAIN_SPEC", default_value = "dev")]
	pub chain: String,

	#[clap(
		long = "tests",
		help = "Comma-separated list of tests to run (if omitted, runs all tests)"
	)]
	pub tests: Option<String>,
}
