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
	moonbase_weights, Balances, BridgeMessages, BridgeXcmOver, MessageQueue, PolkadotXcm, Runtime,
	RuntimeEvent, RuntimeHoldReason,
};
use alloc::collections::btree_set::BTreeSet;
use bp_parachains::SingleParaStoredHeaderDataBuilder;
use bridge_hub_common::xcm_version::XcmVersionOfDestAndRemoteBridge;
use frame_support::pallet_prelude::PalletInfoAccess;
use frame_support::traits::{Contains, Everything};
use frame_support::{parameter_types, traits::ConstU32};
use frame_system::{EnsureNever, EnsureRoot};
use moonbeam_core_primitives::{AccountId, Balance};
use moonbeam_runtime_common::bridge::{CongestionManager, LocalBlobDispatcher};
use pallet_xcm_bridge::XcmAsPlainPayload;
use polkadot_parachain::primitives::Sibling;
use sp_core::hex2array;
use xcm::latest::{Junction, Location};
use xcm::prelude::{GlobalConsensus, InteriorLocation, NetworkId, PalletInstance, Parachain};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};

parameter_types! {
	pub MessagesPalletInstance: InteriorLocation = [PalletInstance(<BridgeMessages as PalletInfoAccess>::index() as u8)].into();

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const ParasPalletName: &'static str = bp_westend::PARAS_PALLET_NAME;
	pub const MaxParaHeadDataSize: u32 = bp_westend::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	pub storage BridgeDeposit: Balance = 0;
}

#[cfg(feature = "bridge-stagenet")]
pub type ThisChain = bp_moonbase::stagenet::Stagenet;
#[cfg(feature = "bridge-stagenet")]
pub type BridgedChain = bp_moonbase::betanet::Betanet;
#[cfg(not(feature = "bridge-stagenet"))]
pub type ThisChain = bp_moonbase::betanet::Betanet;
#[cfg(not(feature = "bridge-stagenet"))]
pub type BridgedChain = bp_moonbase::stagenet::Stagenet;

#[cfg(feature = "bridge-stagenet")]
parameter_types! {
	pub SourceParachain: Junction = Parachain(bp_moonbase::stagenet::PARACHAIN_ID);
	pub TargetParachain: Junction = Parachain(bp_moonbase::betanet::PARACHAIN_ID);
	pub SourceGlobalConsensusNetwork: NetworkId = NetworkId::ByGenesis(hex2array!("64d25a5d58d8d330b8804103e6452be6258ebfd7c4f4c1294835130e75628401"));
	pub TargetGlobalConsensusNetwork: NetworkId = NetworkId::ByGenesis(hex2array!("e1ea3ab1d46ba8f4898b6b4b9c54ffc05282d299f89e84bd0fd08067758c9443"));

	pub TargetGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(TargetGlobalConsensusNetwork::get())]
	);

	pub TargetBridgeLocation: Location = Location::new(
		2,
		[
			GlobalConsensus(TargetGlobalConsensusNetwork::get()),
			TargetParachain::get()
		]
	);

	pub UniversalAliases: BTreeSet<(Location, Junction)> = BTreeSet::from_iter(
		alloc::vec![
			(SelfLocation::get(), GlobalConsensus(TargetGlobalConsensusNetwork::get()))
		]
	);
}

#[cfg(not(feature = "bridge-stagenet"))]
parameter_types! {
	pub SourceParachain: Junction = Parachain(bp_moonbase::betanet::PARACHAIN_ID);
	pub TargetParachain: Junction = Parachain(bp_moonbase::stagenet::PARACHAIN_ID);
	pub SourceGlobalConsensusNetwork: NetworkId = NetworkId::ByGenesis(hex2array!("e1ea3ab1d46ba8f4898b6b4b9c54ffc05282d299f89e84bd0fd08067758c9443"));
	pub TargetGlobalConsensusNetwork: NetworkId = NetworkId::ByGenesis(hex2array!("64d25a5d58d8d330b8804103e6452be6258ebfd7c4f4c1294835130e75628401"));
	pub TargetGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(TargetGlobalConsensusNetwork::get())]
	);

	pub TargetBridgeLocation: Location = Location::new(
		2,
		[
			GlobalConsensus(TargetGlobalConsensusNetwork::get()),
			TargetParachain::get()
		]
	);

	pub UniversalAliases: BTreeSet<(Location, Junction)> = BTreeSet::from_iter(
		alloc::vec![
			(SelfLocation::get(), GlobalConsensus(TargetGlobalConsensusNetwork::get()))
		]
	);
}

impl Contains<(Location, Junction)> for UniversalAliases {
	fn contains(alias: &(Location, Junction)) -> bool {
		UniversalAliases::get().contains(alias)
	}
}

/// Add GRANDPA bridge pallet to track the relay chain finality.
pub type BridgeGrandpaInstance = pallet_bridge_grandpa::Instance1;
impl pallet_bridge_grandpa::Config<BridgeGrandpaInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = bp_westend::Westend;
	type MaxFreeHeadersPerBlock = ConstU32<4>;
	type FreeHeadersInterval = ConstU32<5>;
	type HeadersToKeep = RelayChainHeadersToKeep;
	type WeightInfo = moonbase_weights::pallet_bridge_grandpa::WeightInfo<Runtime>;
}

pub type BridgeParachainsInstance = pallet_bridge_parachains::Instance1;
impl pallet_bridge_parachains::Config<BridgeParachainsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgesGrandpaPalletInstance = BridgeGrandpaInstance;
	type ParasPalletName = ParasPalletName;
	type ParaStoredHeaderDataBuilder = SingleParaStoredHeaderDataBuilder<BridgedChain>;
	type HeadsToKeep = ParachainHeadsToKeep;
	type MaxParaHeadDataSize = MaxParaHeadDataSize;
	type WeightInfo = moonbase_weights::pallet_bridge_parachains::WeightInfo<Runtime>;
}

/// Add XCM messages support for Betanet->Stagenet
pub type WithMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ThisChain = ThisChain;
	type BridgedChain = BridgedChain;
	type BridgedHeaderChain =
		pallet_bridge_parachains::ParachainHeaders<Runtime, BridgeParachainsInstance, BridgedChain>;

	type OutboundPayload = XcmAsPlainPayload;
	type InboundPayload = XcmAsPlainPayload;
	type LaneId = bp_messages::HashedLaneId;

	type DeliveryPayments = ();
	type DeliveryConfirmationPayments = (); // Only necessary if we want to reward relayers

	type MessageDispatch = BridgeXcmOver;
	type OnMessagesDelivered = BridgeXcmOver;
	type WeightInfo = moonbase_weights::pallet_bridge_messages::WeightInfo<Runtime>;
}

/// Add support for the export and dispatch of XCM programs withing
/// `WithMessagesInstance`.
pub type XcmBridgeInstance = pallet_xcm_bridge::Instance1;
impl pallet_xcm_bridge::Config<XcmBridgeInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type UniversalLocation = UniversalLocation;
	type BridgedNetwork = TargetGlobalConsensusNetworkLocation;
	type BridgeMessagesPalletInstance = WithMessagesInstance;

	type MessageExportPrice = ();
	type DestinationVersion = XcmVersionOfDestAndRemoteBridge<PolkadotXcm, TargetBridgeLocation>;

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
	type LocalXcmChannelManager = CongestionManager<Runtime>;
	// Dispatching inbound messages from the bridge and managing congestion with the local
	// receiving/destination chain
	type BlobDispatcher =
		LocalBlobDispatcher<MessageQueue, UniversalLocation, MessagesPalletInstance>;
}
