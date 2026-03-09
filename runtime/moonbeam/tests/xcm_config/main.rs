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

//! XCM Configuration Tests (Level 1)
//!
//! These tests verify the XCM configuration components in isolation using
//! the real Moonbeam runtime configuration. They test:
//!
//! - Barriers: Which XCM messages are allowed to execute
//! - Reserves: Which assets are recognized as reserve assets
//! - Traders: How XCM fees are charged
//! - Transactors: How assets are deposited/withdrawn
//! - Location converters: How locations map to accounts
//! - Weigher: How XCM messages are weighed

#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

mod xcm_common;

mod barriers;
mod location;
mod reserves;
mod traders;
mod transactors;
mod weigher;
