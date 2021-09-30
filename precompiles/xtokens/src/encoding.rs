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
use precompile_utils::{error, Bytes, EvmDataReader};

use frame_support::ensure;
use sp_std::vec::Vec;
use xcm::v1::{Junction, Junctions, NetworkId};

// Encoder trait for xcm types
pub trait Encoder {
	type EncodingType;
	fn to_encoded(&self) -> Self::EncodingType;
	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError>
	where
		Self: Sized;
}

// Implementation of the encoder trait for NetworkId
// Each NetworkId variant is represented as bytes
// The first byte represents the enum variant to be used
// The rest of the bytes (if any), represent the additional data that such enum variant requires
// In this case, only Named requies additional non-bounded data.
// In such a case, since NetworkIds will be appended at the end, we will read the buffer until the
// end to recover the name
impl Encoder for NetworkId {
	type EncodingType = Vec<u8>;
	fn to_encoded(&self) -> Self::EncodingType {
		let mut encoded: Vec<u8> = Vec::new();
		match self.clone() {
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
	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError> {
		ensure!(encoded.len() > 0, error("Junctions cannot be empty"));
		let mut encoded_network_id = EvmDataReader::new(&encoded);

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
}

// Implementation of the encoder type for Junction
// Each Junction is represented as Bytes.
// The first byte represents the enum variant to be used
// The rest of the bytes (if any), represent the additional data that such enum variant requires
// Example: vec![0, 0, 0, 0, 1] would represent Junction::Parachain(1u32)

// NetworkId encodings, if needed, are appended at the end.

impl Encoder for Junction {
	type EncodingType = Vec<u8>;
	fn to_encoded(&self) -> Self::EncodingType {
		let mut encoded: Vec<u8> = Vec::new();
		match self.clone() {
			Junction::Parachain(para_id) => {
				encoded.push(0u8);
				encoded.append(&mut para_id.to_be_bytes().to_vec());
				encoded
			}
			Junction::AccountId32 { network, id } => {
				encoded.push(1u8);
				encoded.append(&mut id.to_vec());
				encoded.append(&mut network.to_encoded());
				encoded
			}
			Junction::AccountIndex64 { network, index } => {
				encoded.push(2u8);
				encoded.append(&mut index.to_be_bytes().to_vec());
				encoded.append(&mut network.to_encoded());
				encoded
			}
			Junction::AccountKey20 { network, key } => {
				encoded.push(3u8);
				encoded.append(&mut key.to_vec());
				encoded.append(&mut network.to_encoded());
				encoded
			}
			Junction::PalletInstance(intance) => {
				encoded.push(4u8);
				encoded.append(&mut intance.to_be_bytes().to_vec());
				encoded
			}
			Junction::GeneralIndex(id) => {
				encoded.push(5u8);
				encoded.append(&mut id.to_be_bytes().to_vec());
				encoded
			}
			Junction::GeneralKey(mut key) => {
				encoded.push(6u8);
				encoded.append(&mut key);
				encoded
			}
			Junction::OnlyChild => {
				encoded.push(7u8);
				encoded
			}
			_ => todo!(),
		}
	}

	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError> {
		ensure!(encoded.len() > 0, error("Junctions cannot be emptyt"));
		let mut encoded_junction = EvmDataReader::new(&encoded);

		let enum_selector = encoded_junction.read_raw_bytes(1)?;

		match enum_selector[0] {
			0 => {
				let mut data: [u8; 4] = Default::default();
				data.copy_from_slice(&encoded_junction.read_raw_bytes(4)?);
				let para_id = u32::from_be_bytes(data);
				Ok(Junction::Parachain(para_id))
			}
			1 => {
				let mut account: [u8; 32] = Default::default();
				account.copy_from_slice(&encoded_junction.read_raw_bytes(32)?);

				Ok(Junction::AccountId32 {
					network: NetworkId::from_encoded(encoded_junction.read_till_end()?.to_vec())?,
					id: account,
				})
			}
			2 => {
				let mut index: [u8; 8] = Default::default();
				index.copy_from_slice(&encoded_junction.read_raw_bytes(8)?);
				// Now we read the network
				Ok(Junction::AccountIndex64 {
					network: NetworkId::from_encoded(encoded_junction.read_till_end()?.to_vec())?,
					index: u64::from_be_bytes(index),
				})
			}
			3 => {
				let mut account: [u8; 20] = Default::default();
				account.copy_from_slice(&encoded_junction.read_raw_bytes(20)?);

				Ok(Junction::AccountKey20 {
					network: NetworkId::from_encoded(encoded_junction.read_till_end()?.to_vec())?,
					key: account,
				})
			}
			4 => Ok(Junction::PalletInstance(
				encoded_junction.read_raw_bytes(1)?[0],
			)),
			5 => {
				let mut general_index: [u8; 16] = Default::default();
				general_index.copy_from_slice(&encoded_junction.read_raw_bytes(16)?);
				Ok(Junction::GeneralIndex(u128::from_be_bytes(general_index)))
			}
			6 => Ok(Junction::GeneralKey(
				encoded_junction.read_till_end()?.to_vec(),
			)),
			7 => Ok(Junction::OnlyChild),
			_ => Err(error("No selector for this")),
		}
	}
}

// Implementation of the encoder type for Junctions

// Each Junction is represented as Bytes, like we have encoded above
// The number of junctions represents the enum variant
// e.g., if Vec<Bytes> is length 1 then we know we have one junction,
// i.e., we need to use Junctions::X1

// For now we only encode up to 4 junctions, which should be sufficient for token transfers
// between parachains
impl Encoder for Junctions {
	type EncodingType = Vec<Bytes>;
	fn to_encoded(&self) -> Self::EncodingType {
		let encoded: Vec<Bytes> = self
			.iter()
			.map(|junction| junction.to_encoded().as_slice().into())
			.collect();
		encoded
	}
	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError> {
		match encoded.len() {
			0 => Ok(Junctions::Here),
			1 => {
				let first_junction = Junction::from_encoded(encoded[0].as_bytes().into())?;
				Ok(Junctions::X1(first_junction))
			}

			2 => {
				let first_junction = Junction::from_encoded(encoded[0].as_bytes().into())?;
				let second_junction = Junction::from_encoded(encoded[1].as_bytes().into())?;
				Ok(Junctions::X2(first_junction, second_junction))
			}
			3 => {
				let first_junction = Junction::from_encoded(encoded[0].as_bytes().into())?;
				let second_junction = Junction::from_encoded(encoded[1].as_bytes().into())?;
				let third_junction = Junction::from_encoded(encoded[2].as_bytes().into())?;

				Ok(Junctions::X3(
					first_junction,
					second_junction,
					third_junction,
				))
			}
			4 => {
				let first_junction = Junction::from_encoded(encoded[0].as_bytes().into())?;
				let second_junction = Junction::from_encoded(encoded[1].as_bytes().into())?;
				let third_junction = Junction::from_encoded(encoded[2].as_bytes().into())?;
				let fourth_junction = Junction::from_encoded(encoded[3].as_bytes().into())?;

				Ok(Junctions::X4(
					first_junction,
					second_junction,
					third_junction,
					fourth_junction,
				))
			}
			_ => Err(error("Provided more than 9 arguments for multilocation")),
		}
	}
}
