// Copyright 2019-2025 Moonbeam Foundation.
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

//! Chain definitions for XCM integration tests.
//!
//! This module contains the chain setups for the xcm-simulator network:
//! - Moonbeam: Uses the real Moonbeam runtime
//! - Relay: Minimal Polkadot relay chain mock
//! - AssetHub: Minimal Asset Hub parachain mock

pub mod asset_hub_mock;
pub mod moonbeam;
pub mod relay_mock;
