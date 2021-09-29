use evm::ExitError;
use precompile_utils::{error, Bytes, EvmDataReader};

use frame_support::ensure;
use sp_std::vec::Vec;
use xcm::v1::{Junction, Junctions, MultiLocation, NetworkId};
pub trait Encoder {
	type EncodingType;
	fn to_encoded(&self) -> Self::EncodingType;
	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError>
	where
		Self: Sized;
}

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

impl Encoder for Junctions {
	type EncodingType = Vec<Bytes>;
	fn to_encoded(&self) -> Self::EncodingType {
		let encoded: Vec<Bytes> = self
			.iter()
			.map(|junction| junction.to_encoded().into())
			.collect();
		encoded
	}
	fn from_encoded(encoded: Self::EncodingType) -> Result<Self, ExitError> {
		match encoded.len() {
			0 => Ok(Junctions::Here),
			1 => {
				let first_junction = Junction::from_encoded(encoded[0].clone().into())?;
				Ok(Junctions::X1(first_junction))
			}

			2 => {
				let first_junction = Junction::from_encoded(encoded[0].clone().into())?;
				let second_junction = Junction::from_encoded(encoded[1].clone().into())?;
				Ok(Junctions::X2(first_junction, second_junction))
			}
			3 => {
				let first_junction = Junction::from_encoded(encoded[0].clone().into())?;
				let second_junction = Junction::from_encoded(encoded[1].clone().into())?;
				let third_junction = Junction::from_encoded(encoded[2].clone().into())?;

				Ok(Junctions::X3(
					first_junction,
					second_junction,
					third_junction,
				))
			}
			4 => {
				let first_junction = Junction::from_encoded(encoded[0].clone().into())?;
				let second_junction = Junction::from_encoded(encoded[1].clone().into())?;
				let third_junction = Junction::from_encoded(encoded[2].clone().into())?;
				let fourth_junction = Junction::from_encoded(encoded[3].clone().into())?;

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
