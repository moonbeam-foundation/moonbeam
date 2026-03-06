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

//! XCM Emulator Integration Tests (Level 2A)
//!
//! Uses the real `moonbeam_runtime` connected to `westend_runtime` as relay
//! and a sibling `moonbeam_runtime` instance. Tests exercise:
//!
//! - Transfers: relay→para, para→relay, para→para (DMP/UMP/XCMP)
//! - Fee collection: treasury receives execution fees, insufficient fees fail
//! - Transact: sovereign transact to relay
//! - HRMP: open, accept, close channels via `pallet_xcm_transactor`
//! - Account sufficiency: fresh accounts receive foreign assets

#![cfg(test)]

mod emulator_asset_hub_tests;
mod emulator_network;
mod emulator_relay;
mod emulator_transact_tests;
mod emulator_transfer_tests;
mod emulator_versioning_tests;
