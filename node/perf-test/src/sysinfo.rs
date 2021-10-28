// Copyright 2019-2021 PureStake Inc.
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

use heim_common::units::{frequency, information};
use heim_cpu;
use heim_memory;
use psutil::{disk, host};
use serde::Serialize;

// Utilities for querying system information, namely:
//
// * Linux kernel info
// * Linux distro info
// * CPU characteristics
// * Available/total memory
// * Disk information

/// Basic system-wide information
#[derive(Default, Debug, Serialize)]
pub struct SystemInfo {
	pub cpu_info: CPUInfo,
	pub mem_info: MemoryInfo,
	pub kernel_version: String,
	pub distro_name: String,
	pub architecture: String,
	// TODO: could include array of PhysicalDiskInfo and/or PartitionInfo
}

/// Memory info
#[derive(Default, Debug, Serialize)]
pub struct MemoryInfo {
	pub total_memory_mb: u64,
	pub available_memory_mb: u64,
	// TODO: would love to include memory speed and physical module info
}

/// CPU info
#[derive(Default, Debug, Serialize)]
pub struct CPUInfo {
	pub cpu_vendor: String,
	pub cpu_model: String,
	pub cpu_base_speed_mhz: u64,
	pub cpu_min_speed_mhz: u64,
	pub num_physical_cores: u64,
	pub num_threads: u64, // TODO: be more technically correct?
}

/// Information about a partition
#[derive(Default, Debug, Serialize)]
pub struct PartitionInfo {
	pub mount_point: String,
	pub device_file: String,
	pub fs_type: String,
	pub total_size_bytes: u64,
	pub available_bytes: u64,
}

pub fn query_system_info() -> Result<SystemInfo, String> {
	use raw_cpuid::CpuId;

	let memory_info = futures::executor::block_on(heim_memory::memory());
	let (mem_total, mem_available) = if let Ok(memory_info) = memory_info {
		let total = memory_info.total().get::<information::megabyte>();
		let available = memory_info.available().get::<information::megabyte>();
		(total, available)
	} else {
		// don't fail the test if the system doesn't report memory info for whatever reason
		(0u64, 0u64)
	};

	let host_info = host::info();

	// TODO: getting info on the boost freq(s) would be very useful
	let cpu_freq = futures::executor::block_on(heim_cpu::frequency());
	let (cpu_min, cpu_max) = if let Ok(cpu_freq) = cpu_freq {
		let min = cpu_freq
			.min()
			.map_or_else(|| 0u64, |freq| freq.get::<frequency::megahertz>());
		let max = cpu_freq
			.max()
			.map_or_else(|| 0u64, |freq| freq.get::<frequency::megahertz>());
		(min, max)
	} else {
		// don't fail the test if the system doesn't report cpu info for whatever reason. this
		// appears to be the case in some virtualization environments.
		// TODO: see if we can work around this
		(0u64, 0u64)
	};

	let cpu_logical_cores = num_cpus::get() as u64;
	let cpu_physical_cores = num_cpus::get_physical() as u64;

	let cpuid = CpuId::new();

	Ok(SystemInfo {
		cpu_info: CPUInfo {
			cpu_vendor: cpuid
				.get_vendor_info()
				.map_or_else(|| String::from("n/a"), |s| s.as_str().into())
				.into(),
			cpu_model: cpuid
				.get_processor_brand_string()
				.map_or_else(|| String::from("n/a"), |s| s.as_str().into())
				.into(),
			cpu_base_speed_mhz: cpu_max,
			cpu_min_speed_mhz: cpu_min,
			num_physical_cores: cpu_physical_cores,
			num_threads: cpu_logical_cores,
		},
		mem_info: MemoryInfo {
			total_memory_mb: mem_total,
			available_memory_mb: mem_available,
		},
		kernel_version: host_info.release().into(),
		distro_name: host_info.version().into(),
		architecture: host_info.architecture().as_str().into(),
	})
}

/// query the partition info corresponding to the given path. the path doesn't need to be an
/// explicit mountpoint; it can be a subdirectory of a mountpoint.
pub fn query_partition_info(path: &std::path::PathBuf) -> Result<PartitionInfo, String> {
	use std::{collections::HashMap, path::Path};

	let canon_path = std::fs::canonicalize(path).expect("Could not deduce canonical path");

	let partitions = disk::partitions_physical().unwrap();

	let mut partitions_map = HashMap::<String, disk::Partition>::new();
	for partition in partitions {
		partitions_map.insert(
			partition
				.mountpoint()
				.to_str()
				.expect("fs paths expected to be valid UTF-8")
				.into(),
			partition,
		);
	}

	// crawl up the parent dirs in path to find a match from our partitions
	let mut ancestors = canon_path.ancestors();
	let partition = ancestors
		.find_map(|dir| {
			let dir_str = dir.to_str().expect("path should be valid UTF-8");
			if let partition = partitions_map.get(dir_str) {
				partition
			} else {
				None
			}
		})
		.expect("Any path should exist under some partition; qed");

	let disk_usage =
		disk::disk_usage(partition.mountpoint()).expect("A partition was found at mountpoint; qed");

	Ok(PartitionInfo {
		mount_point: partition
			.mountpoint()
			.to_str()
			.expect("fs paths expected to be valid UTF-8")
			.into(),
		device_file: partition.device().into(),
		fs_type: partition.filesystem().as_str().into(),
		total_size_bytes: disk_usage.total(),
		available_bytes: disk_usage.free(),
	})
}
