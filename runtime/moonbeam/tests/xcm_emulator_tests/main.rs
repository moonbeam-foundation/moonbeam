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
//! These tests use `xcm-emulator` with the real Moonbeam runtime connected to a
//! minimal mock relay chain. Unlike the `xcm-simulator` tests, the emulator goes
//! through `cumulus_pallet_parachain_system` and `pallet_message_queue` for
//! realistic message dispatch (DMP / UMP / XCMP).
//!
//! The existing xcm-simulator integration tests remain intact as a fallback.

#![cfg(test)]

mod emulator_network;
mod emulator_relay;
mod emulator_transfer_tests;
