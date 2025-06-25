// Copyright 2025 Moonbeam Foundation.
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

extern crate alloc;

use crate::xcm_config::{SelfLocation, UniversalLocation};
use crate::{
	moonriver_weights, Balances, BridgePolkadotMessages, BridgeXcmOverMoonbeam, Get, MessageQueue,
	PolkadotXcm, Runtime, RuntimeEvent, RuntimeHoldReason,
};
use alloc::collections::btree_set::BTreeSet;
use bp_parachains::SingleParaStoredHeaderDataBuilder;
use bridge_hub_common::xcm_version::XcmVersionOfDestAndRemoteBridge;
use core::marker::PhantomData;
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_support::pallet_prelude::PalletInfoAccess;
use frame_support::traits::{Contains, EnqueueMessage, Everything};
use frame_support::{ensure, parameter_types, traits::ConstU32, BoundedVec};
use frame_system::{EnsureNever, EnsureRoot};
use moonbeam_core_primitives::{AccountId, Balance};
use pallet_xcm_bridge::XcmAsPlainPayload;
use parity_scale_codec::{Decode, Encode};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::Vec;
use xcm::latest::{InteriorLocation, Junction, Location, NetworkId, Xcm};
use xcm::opaque::VersionedXcm;
use xcm::prelude::{GlobalConsensus, PalletInstance, Parachain};
use xcm_builder::{
	BridgeMessage, DispatchBlob, DispatchBlobError, ParentIsPreset, SiblingParachainConvertsVia,
};

parameter_types! {
	pub BridgeKusamaToPolkadotMessagesPalletInstance: InteriorLocation = [PalletInstance(<BridgePolkadotMessages as PalletInfoAccess>::index() as u8)].into();
	pub PolkadotGlobalConsensusNetwork: NetworkId = NetworkId::Polkadot;
	pub PolkadotGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(PolkadotGlobalConsensusNetwork::get())]
	);
	pub BridgeMoonbeamLocation: Location = Location::new(
		2,
		[
			GlobalConsensus(PolkadotGlobalConsensusNetwork::get()),
			Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID)
		]
	);
	pub BridgeMoonriverLocation: Location = SelfLocation::get();

	/// Price for every byte of the Kusama -> Polkadot message. Can be adjusted via
	/// governance `set_storage` call.
	pub XcmMoonriverRouterByteFee: Balance = bp_moonbeam::estimate_moonbeam_to_moonriver_byte_fee();

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const PolkadotBridgeParachainPalletName: &'static str = bp_polkadot::PARAS_PALLET_NAME;
	pub const MaxPolkadotParaHeadDataSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	/// Universal aliases
	pub UniversalAliases: BTreeSet<(Location, Junction)> = BTreeSet::from_iter(
		alloc::vec![
			(SelfLocation::get(), GlobalConsensus(PolkadotGlobalConsensusNetwork::get()))
		]
	);

	pub storage BridgeDeposit: Balance = 0;
}

impl Contains<(Location, Junction)> for UniversalAliases {
	fn contains(alias: &(Location, Junction)) -> bool {
		UniversalAliases::get().contains(alias)
	}
}

pub struct LocalBlobDispatcher<MQ, OurPlace, OurPlaceBridgeInstance>(
	PhantomData<(MQ, OurPlace, OurPlaceBridgeInstance)>,
);
impl<
		MQ: EnqueueMessage<AggregateMessageOrigin>,
		OurPlace: Get<InteriorLocation>,
		OurPlaceBridgeInstance: Get<Option<InteriorLocation>>,
	> DispatchBlob for LocalBlobDispatcher<MQ, OurPlace, OurPlaceBridgeInstance>
{
	fn dispatch_blob(blob: Vec<u8>) -> Result<(), DispatchBlobError> {
		let our_universal = OurPlace::get();
		let our_global = our_universal
			.global_consensus()
			.map_err(|()| DispatchBlobError::Unbridgable)?;
		let BridgeMessage {
			universal_dest,
			message,
		} = Decode::decode(&mut &blob[..]).map_err(|_| DispatchBlobError::InvalidEncoding)?;
		let universal_dest: InteriorLocation = universal_dest
			.try_into()
			.map_err(|_| DispatchBlobError::UnsupportedLocationVersion)?;
		// `universal_dest` is the desired destination within the universe: first we need to check
		// we're in the right global consensus.
		let intended_global = universal_dest
			.global_consensus()
			.map_err(|()| DispatchBlobError::NonUniversalDestination)?;
		ensure!(
			intended_global == our_global,
			DispatchBlobError::WrongGlobal
		);
		// let dest = universal_dest.relative_to(&our_universal);
		let xcm: Xcm<()> = message
			.try_into()
			.map_err(|_| DispatchBlobError::UnsupportedXcmVersion)?;

		let msg: BoundedVec<u8, MQ::MaxMessageLen> = VersionedXcm::V5(xcm)
			.encode()
			.try_into()
			.map_err(|_| DispatchBlobError::InvalidEncoding)?;
		MQ::enqueue_message(
			msg.as_bounded_slice(),
			AggregateMessageOrigin::Here, // The message came from the para-chain itself.
		);

		Ok(())
	}
}

/// Add GRANDPA bridge pallet to track Polkadot relay chain.
pub type BridgeGrandpaPolkadotInstance = pallet_bridge_grandpa::Instance1;
impl pallet_bridge_grandpa::Config<BridgeGrandpaPolkadotInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = bp_polkadot::Polkadot;
	type MaxFreeHeadersPerBlock = ConstU32<4>;
	type FreeHeadersInterval = ConstU32<5>;
	type HeadersToKeep = RelayChainHeadersToKeep;
	type WeightInfo = moonriver_weights::pallet_bridge_grandpa::WeightInfo<Runtime>;
}

/// Add parachain bridge pallet to track Moonbeam parachain.
pub type BridgeMoonbeamInstance = pallet_bridge_parachains::Instance1;
impl pallet_bridge_parachains::Config<BridgeMoonbeamInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgesGrandpaPalletInstance = BridgeGrandpaPolkadotInstance;
	type ParasPalletName = PolkadotBridgeParachainPalletName;
	type ParaStoredHeaderDataBuilder = SingleParaStoredHeaderDataBuilder<bp_moonbeam::Moonbeam>;
	type HeadsToKeep = ParachainHeadsToKeep;
	type MaxParaHeadDataSize = MaxPolkadotParaHeadDataSize;
	type WeightInfo = moonriver_weights::pallet_bridge_parachains::WeightInfo<Runtime>;
}

/// Add XCM messages support for Moonbeam->Moonriver
pub type WithPolkadotMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithPolkadotMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type ThisChain = bp_moonriver::Moonriver;
	type BridgedChain = bp_moonbeam::Moonbeam;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeMoonbeamInstance,
		bp_moonbeam::Moonbeam,
	>;

	type OutboundPayload = XcmAsPlainPayload;
	type InboundPayload = XcmAsPlainPayload;
	type LaneId = bp_moonriver::LaneId;

	type DeliveryPayments = ();
	type DeliveryConfirmationPayments = (); // Only necessary if we want to reward relayers

	type MessageDispatch = BridgeXcmOverMoonbeam;
	type OnMessagesDelivered = BridgeXcmOverMoonbeam;
	type WeightInfo = moonriver_weights::pallet_bridge_messages::WeightInfo<Runtime>;
}

/// Add support for the export and dispatch of XCM programs withing
/// `WithPolkadotMessagesInstance`.
pub type XcmOverPolkadotInstance = pallet_xcm_bridge::Instance1;
impl pallet_xcm_bridge::Config<XcmOverPolkadotInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type UniversalLocation = UniversalLocation;
	type BridgedNetwork = PolkadotGlobalConsensusNetworkLocation;
	type BridgeMessagesPalletInstance = WithPolkadotMessagesInstance;

	type MessageExportPrice = ();
	type DestinationVersion = XcmVersionOfDestAndRemoteBridge<PolkadotXcm, BridgeMoonbeamLocation>;

	type ForceOrigin = EnsureRoot<AccountId>;
	// We don't want to allow creating bridges.
	type OpenBridgeOrigin = EnsureNever<Location>;
	// Converter aligned with `OpenBridgeOrigin`.
	type BridgeOriginAccountIdConverter = (
		ParentIsPreset<AccountId>,
		SiblingParachainConvertsVia<Sibling, AccountId>,
	);

	type BridgeDeposit = BridgeDeposit;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	// Don't require a deposit, since we don't allow opening new bridges.
	type AllowWithoutBridgeDeposit = Everything;

	// We are not exporting messages to bridge hub
	type LocalXcmChannelManager = ();
	// Dispatching inbound messages from the bridge and managing congestion with the local
	// receiving/destination chain
	type BlobDispatcher = LocalBlobDispatcher<
		MessageQueue,
		UniversalLocation,
		BridgeKusamaToPolkadotMessagesPalletInstance,
	>;
}
