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

use crate::xcm_config::{UniversalLocation, XcmRouter};
use crate::{
	moonriver_weights, Balances, BridgePolkadotMessages, BridgeXcmOverMoonbeam, PolkadotXcm,
	Runtime, RuntimeEvent, RuntimeHoldReason,
};
use bp_messages::LegacyLaneId;
use bp_parachains::SingleParaStoredHeaderDataBuilder;
use bridge_hub_common::xcm_version::XcmVersionOfDestAndRemoteBridge;
use frame_support::pallet_prelude::PalletInfoAccess;
use frame_support::{parameter_types, traits::ConstU32};
use frame_system::{EnsureNever, EnsureRoot};
use moonbeam_core_primitives::{AccountId, Balance};
use parachains_common::xcm_config::{AllSiblingSystemParachains, RelayOrOtherSystemParachains};
use polkadot_parachain::primitives::Sibling;
use xcm::latest::{InteriorLocation, Location, NetworkId};
use xcm::prelude::{GlobalConsensus, PalletInstance, Parachain};
use xcm_builder::{BridgeBlobDispatcher, ParentIsPreset, SiblingParachainConvertsVia};

/// Encoded XCM blob. We expect the bridge messages pallet to use this blob type for both inbound
/// and outbound payloads.
pub type XcmAsPlainPayload = sp_std::vec::Vec<u8>;

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

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const PolkadotBridgeParachainPalletName: &'static str = bp_polkadot::PARAS_PALLET_NAME;
	pub const MaxPolkadotParaHeadDataSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	// see the `FEE_BOOST_PER_RELAY_HEADER` constant get the meaning of this value
	pub PriorityBoostPerRelayHeader: u64 = 32_007_814_407_814;

	pub storage BridgeDeposit: Balance = 0;
}

/// Dispatches received XCM messages from other bridge
type FromPolkadotMessageBlobDispatcher = BridgeBlobDispatcher<
	XcmRouter,
	UniversalLocation,
	BridgeKusamaToPolkadotMessagesPalletInstance,
>;

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
pub type WithMoonbeamMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithMoonbeamMessagesInstance> for Runtime {
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
	type LaneId = LegacyLaneId;

	type DeliveryPayments = ();
	type DeliveryConfirmationPayments = (); // Only necessary if we want to reward relayers

	type MessageDispatch = BridgeXcmOverMoonbeam;
	type OnMessagesDelivered = BridgeXcmOverMoonbeam;
	// TODO
	type WeightInfo = pallet_bridge_messages::weights::BridgeWeight<Runtime>;
}

/// Add support for the export and dispatch of XCM programs withing
/// `WithMoonbeamMessagesInstance`.
pub type XcmOverPolkadotInstance = pallet_xcm_bridge_hub::Instance1;
impl pallet_xcm_bridge_hub::Config<XcmOverPolkadotInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type UniversalLocation = UniversalLocation;
	type BridgedNetwork = PolkadotGlobalConsensusNetworkLocation;
	type BridgeMessagesPalletInstance = WithMoonbeamMessagesInstance;

	type MessageExportPrice = ();
	type DestinationVersion = XcmVersionOfDestAndRemoteBridge<PolkadotXcm, BridgeMoonbeamLocation>;

	type ForceOrigin = EnsureRoot<AccountId>;
	// We don't want to allow creating bridges for this instance with `LegacyLaneId`.
	type OpenBridgeOrigin = EnsureNever<Location>;
	// Converter aligned with `OpenBridgeOrigin`.
	type BridgeOriginAccountIdConverter = (
		ParentIsPreset<AccountId>,
		SiblingParachainConvertsVia<Sibling, AccountId>,
	);

	type BridgeDeposit = BridgeDeposit;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	// Do not require deposit from system parachains or relay chain
	type AllowWithoutBridgeDeposit =
		RelayOrOtherSystemParachains<AllSiblingSystemParachains, Runtime>;

	type LocalXcmChannelManager = ();
	type BlobDispatcher = FromPolkadotMessageBlobDispatcher;
}
