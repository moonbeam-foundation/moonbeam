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
use futures::{future::BoxFuture, FutureExt as _};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;

use parity_scale_codec::Encode;
use xcm::latest::prelude::*;

//TODO This was in a separate crate in rpc-core for the ethereum-related RPC endpoints.
// But I'm starting with it all in one to keep things simple. Are there drawbacks to doing this?
/// This RPC interface is used to manually submit XCM messages that will be injected into a
/// parachain-enabled runtime. This allows testing XCM logic in a controlled way in integration
/// tests.
#[rpc(server)]
pub trait ManualXcmApi {
	// Inject a downward message - A message that comes from the relay chain.
	#[rpc(name = "xcm_injectDownwardMessage")]
	fn inject_downward_message(&self, message: Vec<u8>) -> BoxFuture<'static, RpcResult<()>>;

	// Inject an HRMP message - A message that comes from a dedicated channel to a sibling
	// parachain.
	#[rpc(name = "xcm_injectHrmpMessage")]
	fn inject_hrmp_message(
		&self,
		channel: u32, //TODO I think there is a better type for this?
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<()>>;
}

pub struct ManualXcm {
	pub downward_message_channel: flume::Sender<Vec<u8>>,
	pub hrmp_message_channel: flume::Sender<Vec<u8>>,
}

impl ManualXcmApi for ManualXcm {
	fn inject_downward_message(&self, msg: Vec<u8>) -> BoxFuture<'static, RpcResult<()>> {
		let downward_message_channel = self.downward_message_channel.clone();
		async move {
			// For now we shadow the message passed in and always insert this downward transfer
			// TODO rename this method insert_encoded or something. Consider a dedicated method
			// to insert DOT transfers that just takes parameters like beneficiary and amount
			let msg = xcm::VersionedXcm::<()>::V2(Xcm(vec![
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
			.encode();

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
		_channel: u32, //TODO I think there is a better type for this?
		_message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<()>> {
		// let mut requester = self.requester.clone();

		println!("---> Enter");

		async move { todo!() }.boxed()
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
