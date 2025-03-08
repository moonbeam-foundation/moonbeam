// Copyright 2025 Moonbeam foundation
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

/// Macro to trigger a compile-time error if the `on-chain-release-build` feature is enabled.
///
/// This ensures that certain code paths are not compiled when the code is built in an
/// on-chain environment, specifically when the `on-chain-release-build` feature is active.
/// This can be useful for enforcing restrictions or preventing undesirable behavior in on-chain
/// builds, ensuring that critical or forbidden code is excluded at compile time.
///
/// # Usage
/// ```rust
/// moonbeam_runtime_common::fail_to_compile_if_on_chain_build!();
/// ```
#[macro_export]
macro_rules! fail_to_compile_if_on_chain_build {
	() => {
		#[cfg(feature = "on-chain-release-build")]
		compile_error!("Not allowed in on-chain builds.");
	};
}
