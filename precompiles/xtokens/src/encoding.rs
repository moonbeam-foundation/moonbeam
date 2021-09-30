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

//! Encoding of XCM types for solidity

use evm::ExitError;
use precompile_utils::{error, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult};
use sp_core::U256;

use frame_support::ensure;
use sp_std::vec::Vec;
use xcm::v1::{Junction, Junctions, MultiLocation, NetworkId};

// Implementation of the encoder trait for NetworkId
// Each NetworkId variant is represented as bytes
// The first byte represents the enum variant to be used
// The rest of the bytes (if any), represent the additional data that such enum variant requires
// In this case, only Named requies additional non-bounded data.
// In such a case, since NetworkIds will be appended at the end, we will read the buffer until the
// end to recover the name
pub(crate) fn network_id_to_bytes(network_id: NetworkId) -> Vec<u8> {
	let mut encoded: Vec<u8> = Vec::new();
	match network_id.clone() {
		NetworkId::Any => {
			encoded.push(0u8);
			encoded
		}
		NetworkId::Named(mut name) => {
			encoded.push(1u8);
			encoded.append(&mut name);
			encoded
		}
		NetworkId::Polkadot => {
			encoded.push(2u8);
			encoded
		}
		NetworkId::Kusama => {
			encoded.push(3u8);
			encoded
		}
	}
}

pub(crate) fn network_id_from_bytes(encoded_bytes: Vec<u8>) -> Result<NetworkId, ExitError> {
	ensure!(encoded_bytes.len() > 0, error("Junctions cannot be empty"));
	let mut encoded_network_id = EvmDataReader::new(&encoded_bytes);

	let network_selector = encoded_network_id.read_raw_bytes(1)?;

	match network_selector[0] {
		0 => Ok(NetworkId::Any),
		1 => Ok(NetworkId::Named(
			encoded_network_id.read_till_end()?.to_vec(),
		)),
		2 => Ok(NetworkId::Polkadot),
		3 => Ok(NetworkId::Kusama),
		_ => Err(error("Non-valid Network Id")),
	}
}

// Implementation of the encoder type for Junction
// Each Junction is represented as Bytes.
// The first byte represents the enum variant to be used
// The rest of the bytes (if any), represent the additional data that such enum variant requires
// Example: vec![0, 0, 0, 0, 1] would represent Junction::Parachain(1u32)

// NetworkId encodings, if needed, are appended at the end.

// A wrapper to be able to implement here the evmData reader
#[derive(Clone, Eq, PartialEq)]
pub struct JunctionWrapper(Junction);

// Implementation of the encoder type for Junction
// Each Junction is represented as Bytes.
// The first byte represents the enum variant to be used
// The rest of the bytes (if any), represent the additional data that such enum variant requires
// Example: vec![0, 0, 0, 0, 1] would represent Junction::Parachain(1u32)
impl From<Junction> for JunctionWrapper {
	fn from(junction: Junction) -> Self {
		JunctionWrapper(junction)
	}
}

impl Into<Junction> for JunctionWrapper {
	fn into(self) -> Junction {
		self.0
	}
}

impl EvmData for JunctionWrapper {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let junction = reader.read::<Bytes>()?;
		let junction_bytes = junction.as_bytes();

		ensure!(junction_bytes.len() > 0, error("Junctions cannot be empty"));
		let mut encoded_junction = EvmDataReader::new(&junction_bytes);

		let enum_selector = encoded_junction.read_raw_bytes(1)?;

		match enum_selector[0] {
			0 => {
				let mut data: [u8; 4] = Default::default();
				data.copy_from_slice(&encoded_junction.read_raw_bytes(4)?);
				let para_id = u32::from_be_bytes(data);
				Ok(JunctionWrapper(Junction::Parachain(para_id)))
			}
			1 => {
				let mut account: [u8; 32] = Default::default();
				account.copy_from_slice(&encoded_junction.read_raw_bytes(32)?);

				Ok(JunctionWrapper(Junction::AccountId32 {
					network: network_id_from_bytes(encoded_junction.read_till_end()?.to_vec())?,
					id: account,
				}))
			}
			2 => {
				let mut index: [u8; 8] = Default::default();
				index.copy_from_slice(&encoded_junction.read_raw_bytes(8)?);
				// Now we read the network
				Ok(JunctionWrapper(Junction::AccountIndex64 {
					network: network_id_from_bytes(encoded_junction.read_till_end()?.to_vec())?,
					index: u64::from_be_bytes(index),
				}))
			}
			3 => {
				let mut account: [u8; 20] = Default::default();
				account.copy_from_slice(&encoded_junction.read_raw_bytes(20)?);

				Ok(JunctionWrapper(Junction::AccountKey20 {
					network: network_id_from_bytes(encoded_junction.read_till_end()?.to_vec())?,
					key: account,
				}))
			}
			4 => Ok(JunctionWrapper(Junction::PalletInstance(
				encoded_junction.read_raw_bytes(1)?[0],
			))),
			5 => {
				let mut general_index: [u8; 16] = Default::default();
				general_index.copy_from_slice(&encoded_junction.read_raw_bytes(16)?);
				Ok(JunctionWrapper(Junction::GeneralIndex(
					u128::from_be_bytes(general_index),
				)))
			}
			6 => Ok(JunctionWrapper(Junction::GeneralKey(
				encoded_junction.read_till_end()?.to_vec(),
			))),
			7 => Ok(JunctionWrapper(Junction::OnlyChild)),
			_ => Err(error("No selector for this")),
		}
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut encoded: Vec<u8> = Vec::new();
		let encoded_bytes: Bytes = match value.0 {
			Junction::Parachain(para_id) => {
				encoded.push(0u8);
				encoded.append(&mut para_id.to_be_bytes().to_vec());
				encoded.as_slice().into()
			}
			Junction::AccountId32 { network, id } => {
				encoded.push(1u8);
				encoded.append(&mut id.to_vec());
				encoded.append(&mut network_id_to_bytes(network));
				encoded.as_slice().into()
			}
			Junction::AccountIndex64 { network, index } => {
				encoded.push(2u8);
				encoded.append(&mut index.to_be_bytes().to_vec());
				encoded.append(&mut network_id_to_bytes(network));
				encoded.as_slice().into()
			}
			Junction::AccountKey20 { network, key } => {
				encoded.push(3u8);
				encoded.append(&mut key.to_vec());
				encoded.append(&mut network_id_to_bytes(network));
				encoded.as_slice().into()
			}
			Junction::PalletInstance(intance) => {
				encoded.push(4u8);
				encoded.append(&mut intance.to_be_bytes().to_vec());
				encoded.as_slice().into()
			}
			Junction::GeneralIndex(id) => {
				encoded.push(5u8);
				encoded.append(&mut id.to_be_bytes().to_vec());
				encoded.as_slice().into()
			}
			Junction::GeneralKey(mut key) => {
				encoded.push(6u8);
				encoded.append(&mut key);
				encoded.as_slice().into()
			}
			Junction::OnlyChild => {
				encoded.push(7u8);
				encoded.as_slice().into()
			}
			_ => todo!(),
		};
		EvmData::write(writer, encoded_bytes);
	}
}

// A wrapper to be able to implement here the evmData reader
#[derive(Clone, Eq, PartialEq)]
pub struct JunctionsWrapper(Junctions);

impl From<Junctions> for JunctionsWrapper {
	fn from(junctions: Junctions) -> Self {
		JunctionsWrapper(junctions)
	}
}

impl Into<Junctions> for JunctionsWrapper {
	fn into(self) -> Junctions {
		self.0
	}
}

impl EvmData for JunctionsWrapper {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		// MultiLocations are defined by their number of parents (u8) and
		// Junctions. We are assuming the Junctions are encoded as defined in
		// the encoding module

		// Essentially, they will be a set of bytes specifying the different
		// enum variants
		let junctions_bytes: Vec<JunctionWrapper> = reader.read()?;

		match junctions_bytes.len() {
			0 => Ok(JunctionsWrapper(Junctions::Here)),
			1 => Ok(JunctionsWrapper(Junctions::X1(
				junctions_bytes[0].clone().into(),
			))),

			2 => Ok(JunctionsWrapper(Junctions::X2(
				junctions_bytes[0].clone().into(),
				junctions_bytes[1].clone().into(),
			))),
			3 => Ok(JunctionsWrapper(Junctions::X3(
				junctions_bytes[0].clone().into(),
				junctions_bytes[1].clone().into(),
				junctions_bytes[2].clone().into(),
			))),
			4 => Ok(JunctionsWrapper(Junctions::X4(
				junctions_bytes[0].clone().into(),
				junctions_bytes[1].clone().into(),
				junctions_bytes[2].clone().into(),
				junctions_bytes[3].clone().into(),
			))),
			_ => Err(error("Provided more than 4 arguments for multilocation")),
		}
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let junctions: Junctions = value.into();
		let encoded: Vec<JunctionWrapper> = junctions
			.iter()
			.map(|junction| JunctionWrapper(junction.clone()))
			.collect();
		EvmData::write(writer, encoded);
	}
}

// A wrapper to be able to implement here the evmData reader
#[derive(Clone, Eq, PartialEq)]
pub struct MultiLocationWrapper(MultiLocation);

impl From<MultiLocation> for MultiLocationWrapper {
	fn from(location: MultiLocation) -> Self {
		MultiLocationWrapper(location)
	}
}

impl Into<MultiLocation> for MultiLocationWrapper {
	fn into(self) -> MultiLocation {
		self.0
	}
}

impl EvmData for MultiLocationWrapper {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		// MultiLocations are defined by their number of parents (u8) and
		// Junctions. We are assuming the Junctions are encoded as defined in
		// the encoding module

		// Essentially, they will be a set of bytes specifying the different
		// enum variants

		let num_parents = reader
			.read::<u8>()
			.map_err(|_| error("tried to parse array offset out of bounds"))?;

		let junctions: JunctionsWrapper = reader.read()?;

		Ok(MultiLocationWrapper(MultiLocation {
			parents: num_parents,
			interior: junctions.into(),
		}))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		EvmData::write(writer, U256::from(value.0.parents));
		EvmData::write(writer, JunctionsWrapper(value.0.interior));
	}
}
