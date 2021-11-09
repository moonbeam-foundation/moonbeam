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

use polkadot_core_primitives::{
	BlockNumber as RelayBlockNumber, InboundDownwardMessage, InboundHrmpMessage,
};

//TODO This was in a separate crate in rpc-core for the ethereum-related RPC endpoints.
// But I'm starting with it all in one to keep things simple. Are there drawbacks to doing this?
/// This RPC interface is used to manually submit XCM messages that will be injected into a
/// parachain-enabled runtime. This allows testing XCM logic in a controlled way in integration
/// tests.
#[rpc(server)]
pub trait ManualXcmApi {
	// Inject a downward message - A message that comes from the relay chain.
	#[rpc(name = "xcm_injectDownwardMessage")]
	fn inject_downward_message(
		&self,
		sent_at: RelayBlockNumber,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>>;
	// For now we return bool which indicates some vague notion of success
	// In the future it may be nice to return which block hash this message was
	// injected into. We may need to learn more about how forks are handled to make
	// sure we don't lose/skip xcm messages in when reorgs happen. I'm not worried about that yet.

	// Inject an HRMP message - A message that comes from a dedicated channel to a sibling
	// parachain.
	#[rpc(name = "xcm_injectHrmpMessage")]
	fn inject_hrmp_message(
		&self,
		channel: u32, //TODO I think there is a better type for this?
		sent_at: RelayBlockNumber,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>>;
}

pub struct ManualXcm {
	pub downward_message_channel: flume::Sender<InboundDownwardMessage>,
	pub hrmp_message_channel: flume::Sender<InboundHrmpMessage>,
}

impl ManualXcmApi for ManualXcm {
	fn inject_downward_message(
		&self,
		sent_at: RelayBlockNumber,
		msg: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>> {
		let downward_message_channel = self.downward_message_channel.clone();
		async move {
			let message = InboundDownwardMessage { sent_at, msg };

			// Send the message back to the service where it will be queued up
			// to be injected in to an upcoming block. Also send a channel on which
			// the success message can be sent back.
			downward_message_channel
				.send_async(message)
				.await
				.map_err(|err| internal_err(err))?;

			Ok(true) //TODO Can we just use () here instead of bool?
		}
		.boxed()
	}

	fn inject_hrmp_message(
		&self,
		_channel: u32, //TODO I think there is a better type for this?
		_sent_at: RelayBlockNumber,
		_message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>> {
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
