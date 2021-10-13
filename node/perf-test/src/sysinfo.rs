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

// Utilities for querying system information, namely:
//
// * Linux kernel info
// * Linux distro info
// * CPU characteristics
// * Available/total memory
// * Disk information

/// Basic system-wide information
pub struct SystemInfo {
	pub cpu_info: CPUInfo,
	pub mem_info: MemoryInfo,
	pub kernel_version: String,
	pub distro_name: String,
	// TODO: could include array of PhysicalDiskInfo and/or PartitionInfo
}

/// Memory info
pub struct MemoryInfo {
	pub total_memory_mb: u64,
	pub available_memory_mb: u64,
	// TODO: would love to include memory speed and physical module info
}

/// CPU info
pub struct CPUInfo {
	pub cpu_model: String,
	pub cpu_max_speed_mhz: u64,
	pub cpu_base_speed_mhz: u64,
	pub num_physical_cores: u64,
	pub num_threads: u64, // TODO: be more technically correct?
}

/// Information about a physical disk
pub struct PhysicalDiskInfo {
	pub model_name: String,
	pub device_file: String,
	pub total_size_bytes: u64,
	pub available_bytes: u64,
}

/// Information about a partition
pub struct PartitionInfo {
	pub mount_point: String,
	pub device_file: String,
	pub fs_type: String,
	pub total_size_bytes: u64,
	pub available_bytes: u64,
	pub physical_disk: PhysicalDiskInfo,
}

pub fn query_system_info() -> Result<SystemInfo, String> {
	todo!()
}

/// query the partition info corresponding to the given path. the path doesn't need to be an
/// explicit mountpoint; it can be a subdirectory of a mountpoint.
/// TODO: use std::path::Path or whatever the CLI parameter for --base-path uses
pub fn query_partition_info(path: &str) -> Result<PartitionInfo, String> {
	todo!()
}
