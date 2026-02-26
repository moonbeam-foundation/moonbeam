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

//! XCM Integration Tests (Level 2)
//!
//! These tests verify end-to-end XCM functionality using xcm-simulator
//! with the real Moonbeam runtime connected to mock relay and sibling chains.
//!
//! Test categories:
//! - Transfers: Asset transfers between chains
//! - Transact: Remote execution via XCM
//! - HRMP: Horizontal relay-routed message passing
//! - EVM: XCM interactions with EVM
//! - Fees: Fee calculation and payment
//! - Errors: Error handling and edge cases

#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

mod chains;
mod networks;

mod errors_test;
mod evm_test;
mod fees_test;
mod hrmp_test;
mod transact_test;
mod transfers_test;
