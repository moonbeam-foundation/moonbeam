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

//! Pallet for tracking HRMP channel events and dispatching HRMP channel actions.
//! - maximum one channel per relation is constraint placed by relay `hrmp` pallet by using
//! HrmpChannelId { sender: Id, recipient: Id } as unique key

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
mod set;
pub use pallet::*;

#[pallet]
pub mod pallet {
	use crate::set::OrderedSet;
	use cumulus_primitives::{DownwardMessageHandler, InboundDownwardMessage, ParaId};
	use frame_support::{pallet_prelude::*, traits::Get};
	use frame_system::pallet_prelude::*;
	use sp_std::{convert::TryFrom, prelude::*};
	use xcm::{
		v0::{Junction, MultiLocation, OriginKind, SendXcm, Xcm},
		VersionedXcm,
	};

	/// Pallet for tracking and managing HRMP channels between other parachains
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Moonbeam parachain identifier
		type ParaId: Get<ParaId>;
		/// XCM Sender for sending outgoing messages
		type XcmSender: SendXcm;
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Sent channel open request to parachain \[recipient_para_id\]
		SenderChannelRequested(ParaId),
		/// Sender channel request accepted by parachain \[recipient_para_id\]
		SenderChannelAccepted(ParaId),
		/// Error with received sender channel accepted because one already exists locally
		SenderChannelAlreadyExists(ParaId),
		/// Error with recipient channel requested because request already exists locally
		RecipientChannelAlreadyExists(ParaId),
		/// Error closing sender channel because channel DNE
		CloseSenderChannelDNE(ParaId),
		/// Error closing recipient channel because channel DNE
		CloseRecipientChannelDNE(ParaId),
		/// Received new channel request with self as recipient
		ReceivedRecipientChannelRequest(ParaId),
		/// Accepted channel open request from parachain
		AcceptedChannelRequest(ParaId),
		/// Requested to close the channel with self as sender \[recipient_para_id\]
		RequestedCloseSenderChannel(ParaId),
		/// Requested to close the channel with self as recipient \[sender_para_id\]
		RequestedCloseRecipientChannel(ParaId),
		/// Closed channel with parachain as recipient and self as sender
		ClosedSenderChannel(ParaId),
		/// Closed channel with parachain as sender and self as recipient
		ClosedRecipientChannel(ParaId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Cannot send message from parachain to self
		CannotSendToSelf,
		/// Call to SendXcm failed
		FailedToSendXcm,
		/// Maximum one channel per relation ~ (sender,receiver) and direction matters
		MaxOneChannelPerRelation,
		/// Cannot accept a recipient channel request not in local storage
		RecipientRequestDNE,
		/// Requires existing open channel with self as sender
		NoSenderChannelOpen,
		/// Requires existing open channel with self as recipient
		NoRecipientChannelOpen,
	}

	#[pallet::storage]
	#[pallet::getter(fn recipient_channel_requests)]
	/// Open channel requests on the relay chain to self (recipient) from these parachains (sender)
	pub type RecipientChannelRequests<T: Config> = StorageValue<_, OrderedSet<ParaId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sender_channels)]
	/// Stores all para IDs with which this parachain has opened a channel with self as sender
	pub type SenderChannels<T: Config> = StorageValue<_, OrderedSet<ParaId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn recipient_channels)]
	/// Stores all para IDs with which this parachain has accepted a channel with self as recipient
	pub type RecipientChannels<T: Config> = StorageValue<_, OrderedSet<ParaId>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// Request to open HRMP channel with another parachain
		pub fn open_channel(
			origin: OriginFor<T>,
			recipient: ParaId,
			// temporary until proper Xcm message variant is added with handler logic
			call: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let sender = T::ParaId::get();
			ensure!(sender != recipient, Error::<T>::CannotSendToSelf);
			// TODO: could check if sender has already made the request; SenderChannelRequests vec
			let channels = <SenderChannels<T>>::get();
			ensure!(
				!channels.contains(&recipient),
				Error::<T>::MaxOneChannelPerRelation
			);
			// call `hrmp_init_open_channel` on relay chain
			let message = Xcm::Transact {
				origin_type: OriginKind::Native,
				call,
			};
			// send message to relay chain
			T::XcmSender::send_xcm(
				MultiLocation::X1(Junction::Parachain { id: sender.into() }),
				message,
			)
			.map_err(|_| Error::<T>::FailedToSendXcm)?;
			// emit event
			Self::deposit_event(Event::SenderChannelRequested(recipient));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Accept a request to open HRMP channel
		pub fn accept_channel(
			origin: OriginFor<T>,
			sender: ParaId,
			// temporary until proper Xcm message variant is added with handler logic
			call: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let self_id = T::ParaId::get();
			ensure!(sender != self_id, Error::<T>::CannotSendToSelf);
			// first check if the request even exists (all requests are stored locally)
			let mut requests = <RecipientChannelRequests<T>>::get();
			ensure!(requests.remove(&sender), Error::<T>::RecipientRequestDNE);
			let mut channels = <RecipientChannels<T>>::get();
			ensure!(
				channels.insert(sender),
				Error::<T>::MaxOneChannelPerRelation
			);
			// call `hrmp_accept_open_channel` on relay chain
			let message = Xcm::Transact {
				origin_type: OriginKind::Native,
				call,
			};
			// send message to relay chain
			T::XcmSender::send_xcm(
				MultiLocation::X1(Junction::Parachain { id: self_id.into() }),
				message,
			)
			.map_err(|_| Error::<T>::FailedToSendXcm)?;
			// update recipient channel requests
			<RecipientChannelRequests<T>>::put(requests);
			// update recipient channels storage item
			<RecipientChannels<T>>::put(channels);
			// emit event
			Self::deposit_event(Event::AcceptedChannelRequest(sender));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Close an open HRMP channel with self as sender
		pub fn close_sender_channel(
			origin: OriginFor<T>,
			recipient: ParaId,
			// temporary until hrmp variants added to Xcm
			call: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let self_id = T::ParaId::get();
			ensure!(recipient != self_id, Error::<T>::CannotSendToSelf);
			let channels = <SenderChannels<T>>::get();
			ensure!(
				channels.contains(&recipient),
				Error::<T>::NoSenderChannelOpen
			);
			// call `hrmp_close_channel` on relay chain
			let message = Xcm::Transact {
				origin_type: OriginKind::Native,
				call,
			};
			// send message to relay chain
			T::XcmSender::send_xcm(
				MultiLocation::X1(Junction::Parachain { id: self_id.into() }),
				message,
			)
			.map_err(|_| Error::<T>::FailedToSendXcm)?;
			Self::deposit_event(Event::RequestedCloseSenderChannel(recipient));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Close an open HRMP channel with self as recipient
		pub fn close_recipient_channel(
			origin: OriginFor<T>,
			sender: ParaId,
			// temporary until hrmp variants added to Xcm
			call: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let self_id = T::ParaId::get();
			ensure!(sender != self_id, Error::<T>::CannotSendToSelf);
			let channels = <RecipientChannels<T>>::get();
			ensure!(
				channels.contains(&sender),
				Error::<T>::NoRecipientChannelOpen
			);
			// call is to `hrmp_close_channel` on relay chain
			let message = Xcm::Transact {
				origin_type: OriginKind::Native,
				call,
			};
			// send message to accept the channel request
			T::XcmSender::send_xcm(
				MultiLocation::X1(Junction::Parachain { id: self_id.into() }),
				message,
			)
			.map_err(|_| Error::<T>::FailedToSendXcm)?;
			Self::deposit_event(Event::RequestedCloseRecipientChannel(sender));
			Ok(().into())
		}
	}

	impl<T: Config> DownwardMessageHandler for Module<T> {
		fn handle_downward_message(msg: InboundDownwardMessage) {
			match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
				Ok(Ok(xcm)) => {
					match xcm {
						Xcm::HrmpNewChannelOpenRequest { sender, .. } => {
							let mut channels = <RecipientChannelRequests<T>>::get();
							let sender: ParaId = sender.into();
							// if request from id already exists, not added to requests
							if channels.insert(sender) {
								<RecipientChannelRequests<T>>::put(channels);
								Self::deposit_event(Event::ReceivedRecipientChannelRequest(sender));
							} else {
								// event error
								Self::deposit_event(Event::RecipientChannelAlreadyExists(sender));
							}
						}
						Xcm::HrmpChannelAccepted { recipient } => {
							let mut channels = <SenderChannels<T>>::get();
							let recipient: ParaId = recipient.into();
							// if channel with id already exists, not added to channels
							if channels.insert(recipient) {
								<SenderChannels<T>>::put(channels);
								Self::deposit_event(Event::SenderChannelAccepted(recipient));
							} else {
								// event error
								Self::deposit_event(Event::SenderChannelAlreadyExists(recipient));
							}
						}
						Xcm::HrmpChannelClosing {
							sender, recipient, ..
						} => {
							let self_id = T::ParaId::get();
							let sender: ParaId = sender.into();
							let recipient: ParaId = recipient.into();
							if sender == self_id {
								let mut channels = <SenderChannels<T>>::get();
								// update storage
								if channels.remove(&recipient) {
									<SenderChannels<T>>::put(channels);
									Self::deposit_event(Event::ClosedSenderChannel(recipient));
								} else {
									// event error
									Self::deposit_event(Event::CloseSenderChannelDNE(recipient));
								}
							}
							if recipient == self_id {
								let mut channels = <RecipientChannels<T>>::get();
								// update storage
								if channels.remove(&sender) {
									<RecipientChannels<T>>::put(channels);
									Self::deposit_event(Event::ClosedRecipientChannel(sender));
								} else {
									// event error
									Self::deposit_event(Event::CloseRecipientChannelDNE(sender));
								}
							}
						}
						_ => (),
					}
				}
				Ok(Err(..)) => (),
				Err(..) => (),
			}
		}
	}
}
