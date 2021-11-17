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
use cumulus_primitives_core::ParaId;
use futures::{future::BoxFuture, FutureExt as _};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;

use parity_scale_codec::Encode;
use xcm::latest::prelude::*;

/// This RPC interface is used to manually submit XCM messages that will be injected into a
/// parachain-enabled runtime. This allows testing XCM logic in a controlled way in integration
/// tests.
#[rpc(server)]
pub trait ManualXcmApi {
	/// Inject a downward xcm message - A message that comes from the relay chain.
	/// You may provide an arbitrary message, or if you provide an emtpy byte array,
	/// Then a default message (DOT transfer down to ALITH) will be injected
	#[rpc(name = "xcm_injectDownwardMessage")]
	fn inject_downward_message(&self, message: Vec<u8>) -> BoxFuture<'static, RpcResult<()>>;

	/// Inject an HRMP message - A message that comes from a dedicated channel to a sibling
	/// parachain.
	///
	/// Cumulus Parachain System seems to have a constraint that at most one hrmp message will be
	/// sent on a channel per block. At least that's what this comment implies:
	/// https://github.com/paritytech/cumulus/blob/c308c01b/pallets/parachain-system/src/lib.rs#L204
	/// Neither this RPC, nor the mock inherent data provider make any attempt to enforce this
	/// constraint. In fact, violating it may be useful for testing.
	#[rpc(name = "xcm_injectHrmpMessage")]
	fn inject_hrmp_message(
		&self,
		sender: ParaId,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<()>>;
}

pub struct ManualXcm {
	pub downward_message_channel: flume::Sender<Vec<u8>>,
	pub hrmp_message_channel: flume::Sender<(ParaId, Vec<u8>)>,
}

impl ManualXcmApi for ManualXcm {
	fn inject_downward_message(&self, msg: Vec<u8>) -> BoxFuture<'static, RpcResult<()>> {
		let downward_message_channel = self.downward_message_channel.clone();
		async move {
			// If no message is supplied, inject a default one.
			let msg = if msg.is_empty() {
				xcm::VersionedXcm::<()>::V2(Xcm(vec![
					ReserveAssetDeposited((Parent, 10000000000000).into()),
					ClearOrigin,
					BuyExecution {
						fees: (Parent, 10000000000000).into(),
						weight_limit: Limited(4_000_000_000),
					},
					DepositAsset {
						assets: All.into(),
						max_assets: 1,
						beneficiary: MultiLocation::new(
							0,
							X1(AccountKey20 {
								network: Any,
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
				.map_err(|err| internal_err(err))?;

			Ok(())
		}
		.boxed()
	}

	fn inject_hrmp_message(
		&self,
		sender: ParaId,
		msg: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<()>> {
		let hrmp_message_channel = self.hrmp_message_channel.clone();

		async move {
			// Push the message and sender to the shared channel where they will be queued up
			// to be injected in to an upcoming block.
			hrmp_message_channel
				.send_async((sender, msg))
				.await
				.map_err(|err| internal_err(err))?;

			Ok(())
		}
		.boxed()
	}
}

// This bit cribbed from frontier.
pub fn internal_err<T: ToString>(message: T) -> jsonrpc_core::Error {
	jsonrpc_core::Error {
		code: jsonrpc_core::ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}
