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

use psutil::{disk, host};
use heim_cpu;
use heim_memory;
use heim_common::units::{frequency, information};

// Utilities for querying system information, namely:
//
// * Linux kernel info
// * Linux distro info
// * CPU characteristics
// * Available/total memory
// * Disk information

/// Basic system-wide information
#[derive(Default, Debug)]
pub struct SystemInfo {
	pub cpu_info: CPUInfo,
	pub mem_info: MemoryInfo,
	pub kernel_version: String,
	pub distro_name: String,
	pub architecture: String,
	// TODO: could include array of PhysicalDiskInfo and/or PartitionInfo
}

/// Memory info
#[derive(Default, Debug)]
pub struct MemoryInfo {
	pub total_memory_mb: u64,
	pub available_memory_mb: u64,
	// TODO: would love to include memory speed and physical module info
}

/// CPU info
#[derive(Default, Debug)]
pub struct CPUInfo {
	pub cpu_model: String,
	pub cpu_max_speed_mhz: u64,
	pub cpu_base_speed_mhz: u64,
	pub num_physical_cores: u64,
	pub num_threads: u64, // TODO: be more technically correct?
}

/// Information about a physical disk
#[derive(Default, Debug)]
pub struct PhysicalDiskInfo {
	pub model_name: String,
	pub device_file: String,
	pub total_size_bytes: u64,
	pub available_bytes: u64,
}

/// Information about a partition
#[derive(Default, Debug)]
pub struct PartitionInfo {
	pub mount_point: String,
	pub device_file: String,
	pub fs_type: String,
	pub total_size_bytes: u64,
	pub available_bytes: u64,
	pub physical_disk: PhysicalDiskInfo,
}

pub fn query_system_info() -> Result<SystemInfo, String> {
	let memory_info = futures::executor::block_on(heim_memory::memory())
		.expect("Memory must exist; qed");

	let host_info = host::info();
	dbg!(host_info.clone());

	// TODO: block on multiple futures
	let cpu_freq = futures::executor::block_on(heim_cpu::frequency())
		.expect("CPU must exist; qed");
	dbg!(cpu_freq.current());

	let cpu_logical_cores = futures::executor::block_on(heim_cpu::logical_count())
		.expect("CPU must exist; qed");
	let cpu_physical_cores = futures::executor::block_on(heim_cpu::physical_count())
		.expect("CPU must exist; qed")
		.expect("CPU should report num physical cores");

	Ok(SystemInfo {
		cpu_info: CPUInfo {
			cpu_model: "FIXME".into(),
			cpu_max_speed_mhz: cpu_freq.max()
				.expect("could not get CPU max")
				.get::<frequency::megahertz>(),
			cpu_base_speed_mhz: cpu_freq.min() // TODO: we want base, not min
				.expect("could not get CPU min")
				.get::<frequency::megahertz>(),
			num_physical_cores: cpu_physical_cores,
			num_threads: cpu_logical_cores,
		},
		mem_info: MemoryInfo {
			total_memory_mb: memory_info.total().get::<information::megabyte>(),
			available_memory_mb: memory_info.available().get::<information::megabyte>(),
		},
		kernel_version: host_info.release().into(),
		distro_name: host_info.version().into(),
		architecture: host_info.architecture().as_str().into(),
	})
}

/// query the partition info corresponding to the given path. the path doesn't need to be an
/// explicit mountpoint; it can be a subdirectory of a mountpoint.
/// TODO: use std::path::Path or whatever the CLI parameter for --base-path uses
pub fn query_partition_info(path: &str) -> Result<PartitionInfo, String> {
	use std::{collections::HashMap, path::Path};

	let partitions = disk::partitions_physical().unwrap();

	let mut partitions_map = HashMap::<String, disk::Partition>::new();
	for partition in partitions {
		partitions_map.insert(
			partition.mountpoint().to_str().expect("fs paths expected to be valid UTF-8").into(),
			partition);
	}

	// crawl up the parent dirs in path to find a match from our partitions
	let mut ancestors = Path::new(path).ancestors();
	let partition = ancestors.find_map(|dir| {
		let dir_str = dir.to_str().expect("path should be valid UTF-8");
		if let partition = partitions_map.get(dir_str) {
			partition
		} else {
			None
		}
	}).expect("Any path should exist under some partition; qed");

	let disk_usage = disk::disk_usage(partition.mountpoint())
		.expect("A partition was found at mountpoint; qed");

	Ok(PartitionInfo {
		mount_point: partition.mountpoint()
			.to_str()
			.expect("fs paths expected to be valid UTF-8")
			.into(),
		device_file: partition.device().into(),
		fs_type: partition.filesystem().as_str().into(),
		total_size_bytes: disk_usage.total(),
		available_bytes: disk_usage.free(),
		physical_disk: Default::default(), // TODO
	})
}
