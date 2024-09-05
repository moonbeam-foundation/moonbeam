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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use substrate_wasm_builder::WasmBuilder;

#[cfg(not(feature = "metadata-hash"))]
fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}

#[cfg(feature = "metadata-hash")]
fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.enable_metadata_hash("MOVR", 18)
		.build()
}
