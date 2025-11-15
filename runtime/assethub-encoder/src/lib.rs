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

//! Encoder for AssetHub runtimes
//!
//! This crate provides chain-specific indices for AssetHub parachains across different
//! relay chains (Polkadot, Kusama, Westend). These indices are required for properly
//! encoding calls to be executed on AssetHub via XCM.
//!
//! ## Index Verification
//!
//! Indices MUST be verified against the actual AssetHub runtime for each network before use.
//! Incorrect indices will cause transaction failures. Use the following command to extract
//! indices from AssetHub metadata:
//!
//! ```bash
//! # For Polkadot AssetHub
//! subxt metadata --url wss://polkadot-asset-hub-rpc.polkadot.io:443
//!
//! # For Kusama AssetHub
//! subxt metadata --url wss://kusama-asset-hub-rpc.polkadot.io:443
//!
//! # For Westend AssetHub (testnet)
//! subxt metadata --url wss://westend-asset-hub-rpc.polkadot.io:443
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod kusama;
pub mod polkadot;
pub mod westend;
