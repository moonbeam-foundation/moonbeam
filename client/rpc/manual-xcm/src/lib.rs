// Copyright 2019-2022 PureStake Inc.
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
use cumulus_primitives_core::ParaId;
use cumulus_primitives_core::XcmpMessageFormat;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use parity_scale_codec::Encode;
use xcm::latest::prelude::*;
use xcm::opaque::lts::Weight;
use xcm_primitives::DEFAULT_PROOF_SIZE;

/// This RPC interface is used to manually submit XCM messages that will be injected into a
/// parachain-enabled runtime. This allows testing XCM logic in a controlled way in integration
/// tests.
#[rpc(server)]
#[jsonrpsee::core::async_trait]
pub trait ManualXcmApi {
	/// Inject a downward xcm message - A message that comes from the relay chain.
	/// You may provide an arbitrary message, or if you provide an emtpy byte array,
	/// Then a default message (DOT transfer down to ALITH) will be injected
	#[method(name = "xcm_injectDownwardMessage")]
	async fn inject_downward_message(&self, message: Vec<u8>) -> RpcResult<()>;

	/// Inject an HRMP message - A message that comes from a dedicated channel to a sibling
	/// parachain.
	///
	/// Cumulus Parachain System seems to have a constraint that at most one hrmp message will be
	/// sent on a channel per block. At least that's what this comment implies:
	/// https://github.com/paritytech/cumulus/blob/c308c01b/pallets/parachain-system/src/lib.rs#L204
	/// Neither this RPC, nor the mock inherent data provider make any attempt to enforce this
	/// constraint. In fact, violating it may be useful for testing.
	/// The method accepts a sending paraId and a bytearray representing an arbitrary message as
	/// parameters. If you provide an emtpy byte array, then a default message representing a
	/// transfer of the sending paraId's native token will be injected.
	#[method(name = "xcm_injectHrmpMessage")]
	async fn inject_hrmp_message(&self, sender: ParaId, message: Vec<u8>) -> RpcResult<()>;
}

pub struct ManualXcm {
	pub downward_message_channel: flume::Sender<Vec<u8>>,
	pub hrmp_message_channel: flume::Sender<(ParaId, Vec<u8>)>,
}

#[jsonrpsee::core::async_trait]
impl ManualXcmApiServer for ManualXcm {
	async fn inject_downward_message(&self, msg: Vec<u8>) -> RpcResult<()> {
		let downward_message_channel = self.downward_message_channel.clone();
		// If no message is supplied, inject a default one.
		let msg = if msg.is_empty() {
			xcm::VersionedXcm::<()>::V3(Xcm(vec![
				ReserveAssetDeposited((Parent, 10000000000000u128).into()),
				ClearOrigin,
				BuyExecution {
					fees: (Parent, 10000000000000u128).into(),
					weight_limit: Limited(Weight::from_parts(4_000_000_000u64, DEFAULT_PROOF_SIZE)),
				},
				DepositAsset {
					assets: AllCounted(1).into(),
					beneficiary: MultiLocation::new(
						0,
						X1(AccountKey20 {
							network: None,
							key: hex_literal::hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"),
						}),
					),
				},
			]))
			.encode()
		} else {
			msg
		};

		// Push the message to the shared channel where it will be queued up
		// to be injected in to an upcoming block.
		downward_message_channel
			.send_async(msg)
			.await
			.map_err(|err| internal_err(err.to_string()))?;

		Ok(())
	}

	async fn inject_hrmp_message(&self, sender: ParaId, msg: Vec<u8>) -> RpcResult<()> {
		let hrmp_message_channel = self.hrmp_message_channel.clone();

		// If no message is supplied, inject a default one.
		let msg = if msg.is_empty() {
			let mut mes = XcmpMessageFormat::ConcatenatedVersionedXcm.encode();
			mes.append(
				&mut (xcm::VersionedXcm::<()>::V3(Xcm(vec![
					ReserveAssetDeposited(
						((Parent, Parachain(sender.into())), 10000000000000u128).into(),
					),
					ClearOrigin,
					BuyExecution {
						fees: ((Parent, Parachain(sender.into())), 10000000000000u128).into(),
						weight_limit: Limited(Weight::from_parts(
							4_000_000_000u64,
							DEFAULT_PROOF_SIZE,
						)),
					},
					DepositAsset {
						assets: AllCounted(1).into(),
						beneficiary: MultiLocation::new(
							0,
							X1(AccountKey20 {
								network: None,
								key: hex_literal::hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"),
							}),
						),
					},
				]))
				.encode()),
			);
			mes
		} else {
			msg
		};

		// Push the message to the shared channel where it will be queued up
		// to be injected in to an upcoming block.
		hrmp_message_channel
			.send_async((sender, msg))
			.await
			.map_err(|err| internal_err(err.to_string()))?;

		Ok(())
	}
}

// This bit cribbed from frontier.
pub fn internal_err<T: AsRef<str>>(message: T) -> jsonrpsee::core::Error {
	jsonrpsee::core::Error::Call(jsonrpsee::types::error::CallError::Custom(
		jsonrpsee::types::error::ErrorObject::borrowed(
			jsonrpsee::types::error::INTERNAL_ERROR_CODE,
			&message,
			None,
		)
		.into_owned(),
	))
}
