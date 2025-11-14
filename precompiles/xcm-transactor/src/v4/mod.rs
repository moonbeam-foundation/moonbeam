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

//! XCM Transactor Precompile V4 - AssetHub Support
//!
//! V4 adds support for AssetHub as a transaction destination while maintaining
//! full backwards compatibility with V3. The transactor parameter now accepts:
//! - 0 = Relay Chain (Polkadot/Kusama/Westend)
//! - 1 = AssetHub system parachain
//!
//! Key Features:
//! - AssetHub staking operations via delegated staking
//! - Maintain existing V3 functionality
//! - Same function signatures as V3 (just enhanced documentation)

// V4 shares the same implementation as V3 since the function signatures are identical
// The main difference is enhanced documentation and AssetHub support at the pallet level
pub use crate::v3::*;

/// Type alias for V4 - same implementation as V3 with AssetHub support
pub type XcmTransactorPrecompileV4<Runtime> = crate::v3::XcmTransactorPrecompileV3<Runtime>;
