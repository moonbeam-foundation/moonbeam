// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

benchmarks! {
	// VALUES
	get_u64_value {
	}: _(RawOrigin::None)
	verify {}

	put_u64_value {
	}: _(RawOrigin::None, 1u64)
	verify {}

	get_put_u64_value {
	}: _(RawOrigin::None, 1u64)
	verify {}

	emit_u64_value_event {
	}: _(RawOrigin::None)
	verify {}

	get_emit_u64_value_event {
	}: _(RawOrigin::None)
	verify {}

	get_u64_option {
	}: _(RawOrigin::None)
	verify {}

	put_u64_option {
	}: _(RawOrigin::None, 1u64)
	verify {}

	get_put_u64_option {
	}: _(RawOrigin::None, 1u64)
	verify {}

	emit_u64_option_event {
	}: _(RawOrigin::None)
	verify {}

	get_emit_u64_option_event {
	}: _(RawOrigin::None)
	verify {}

	// MAPS
	get_u64_map_value {
	}: _(RawOrigin::None, 1u64)
	verify {}

	put_u64_map_value {
	}: _(RawOrigin::None, 1u64, 2u64)
	verify {}

	get_put_u64_map_value {
	}: _(RawOrigin::None, 1u64, 2u64)
	verify {}

	// DOUBLE MAPS
	get_u64_double_map_value {
	}: _(RawOrigin::None, 1u64, 1u64)
	verify {}

	put_u64_double_map_value {
	}: _(RawOrigin::None, 1u64, 1u64, 2u64)
	verify {}

	get_put_u64_double_map_value {
	}: _(RawOrigin::None, 1u64, 1u64, 2u64)
	verify {}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
