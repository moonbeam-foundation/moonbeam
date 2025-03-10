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
	moonbeam_weights, Balances, BridgeKusamaMessages, BridgeXcmOverMoonriver, PolkadotXcm, Runtime,
	RuntimeEvent, RuntimeHoldReason,
};
use bp_messages::LegacyLaneId;
use bp_parachains::SingleParaStoredHeaderDataBuilder;
use bridge_hub_common::xcm_version::XcmVersionOfDestAndRemoteBridge;
use frame_support::pallet_prelude::PalletInfoAccess;
use frame_support::{parameter_types, traits::ConstU32};
use frame_system::{EnsureNever, EnsureRoot};
use moonbeam_core_primitives::{AccountId, Balance};
use pallet_xcm_bridge_hub::XcmAsPlainPayload;
use parachains_common::xcm_config::{AllSiblingSystemParachains, RelayOrOtherSystemParachains};
use polkadot_parachain::primitives::Sibling;
use xcm::latest::Location;
use xcm::prelude::{GlobalConsensus, InteriorLocation, NetworkId, PalletInstance, Parachain};
use xcm_builder::{BridgeBlobDispatcher, ParentIsPreset, SiblingParachainConvertsVia};

parameter_types! {
	pub BridgePolkadotToKusamaMessagesPalletInstance: InteriorLocation = [PalletInstance(<BridgeKusamaMessages as PalletInfoAccess>::index() as u8)].into();
	pub KusamaGlobalConsensusNetwork: NetworkId = NetworkId::Kusama;
	pub KusamaGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(KusamaGlobalConsensusNetwork::get())]
	);
	pub BridgeMoonriverLocation: Location = Location::new(
		2,
		[
			GlobalConsensus(KusamaGlobalConsensusNetwork::get()),
			Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID)
		]
	);

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const KusamaBridgeParachainPalletName: &'static str = bp_kusama::PARAS_PALLET_NAME;
	pub const MaxKusamaParaHeadDataSize: u32 = bp_kusama::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	// see the `FEE_BOOST_PER_RELAY_HEADER` constant get the meaning of this value
	pub PriorityBoostPerRelayHeader: u64 = 32_007_814_407_814;


	pub storage BridgeDeposit: Balance = 0;
}

/// Dispatches received XCM messages from other bridge
type FromKusamaMessageBlobDispatcher = BridgeBlobDispatcher<
	XcmRouter,
	UniversalLocation,
	BridgePolkadotToKusamaMessagesPalletInstance,
>;

/// Add GRANDPA bridge pallet to track Kusama relay chain.
pub type BridgeGrandpaKusamaInstance = pallet_bridge_grandpa::Instance1;
impl pallet_bridge_grandpa::Config<BridgeGrandpaKusamaInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = bp_kusama::Kusama;
	type MaxFreeHeadersPerBlock = ConstU32<4>;
	type FreeHeadersInterval = ConstU32<5>;
	type HeadersToKeep = RelayChainHeadersToKeep;
	type WeightInfo = moonbeam_weights::pallet_bridge_grandpa::WeightInfo<Runtime>;
}

/// Add parachain bridge pallet to track Moonriver parachain.
pub type BridgeMoonriverInstance = pallet_bridge_parachains::Instance1;
impl pallet_bridge_parachains::Config<BridgeMoonriverInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgesGrandpaPalletInstance = BridgeGrandpaKusamaInstance;
	type ParasPalletName = KusamaBridgeParachainPalletName;
	type ParaStoredHeaderDataBuilder = SingleParaStoredHeaderDataBuilder<bp_moonriver::Moonriver>;
	type HeadsToKeep = ParachainHeadsToKeep;
	type MaxParaHeadDataSize = MaxKusamaParaHeadDataSize;
	type WeightInfo = moonbeam_weights::pallet_bridge_parachains::WeightInfo<Runtime>;
}

/// Add XCM messages support for Moonbeam->Moonriver
pub type WithMoonriverMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithMoonriverMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type ThisChain = bp_moonbeam::Moonbeam;
	type BridgedChain = bp_moonriver::Moonriver;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeMoonriverInstance,
		bp_moonriver::Moonriver,
	>;

	type OutboundPayload = XcmAsPlainPayload;
	type InboundPayload = XcmAsPlainPayload;
	type LaneId = LegacyLaneId;

	type DeliveryPayments = ();
	type DeliveryConfirmationPayments = (); // Only necessary if we want to reward relayers

	type MessageDispatch = BridgeXcmOverMoonriver;
	type OnMessagesDelivered = BridgeXcmOverMoonriver;
	// TODO
	type WeightInfo = pallet_bridge_messages::weights::BridgeWeight<Runtime>;
}

/// Add support for the export and dispatch of XCM programs withing
/// `WithMoonriverMessagesInstance`.
pub type XcmOverKusamaInstance = pallet_xcm_bridge_hub::Instance1;
impl pallet_xcm_bridge_hub::Config<XcmOverKusamaInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type UniversalLocation = UniversalLocation;
	type BridgedNetwork = KusamaGlobalConsensusNetworkLocation;
	type BridgeMessagesPalletInstance = WithMoonriverMessagesInstance;

	type MessageExportPrice = ();
	type DestinationVersion = XcmVersionOfDestAndRemoteBridge<PolkadotXcm, BridgeMoonriverLocation>;

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
	type BlobDispatcher = FromKusamaMessageBlobDispatcher;
}
