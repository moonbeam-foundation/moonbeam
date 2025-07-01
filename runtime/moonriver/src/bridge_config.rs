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
	moonriver_weights, Balances, BridgePolkadotMessages, BridgeXcmOverMoonbeam, MessageQueue,
	PolkadotXcm, Runtime, RuntimeEvent, RuntimeHoldReason,
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
use xcm::latest::{InteriorLocation, Junction, Location, NetworkId};
use xcm::prelude::{GlobalConsensus, PalletInstance};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};

parameter_types! {
	pub BridgeKusamaToPolkadotMessagesPalletInstance: InteriorLocation = [PalletInstance(<BridgePolkadotMessages as PalletInfoAccess>::index() as u8)].into();
	pub PolkadotGlobalConsensusNetwork: NetworkId = NetworkId::Polkadot;
	pub PolkadotGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(PolkadotGlobalConsensusNetwork::get())]
	);

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const PolkadotBridgeParachainPalletName: &'static str = bp_polkadot::PARAS_PALLET_NAME;
	pub const MaxPolkadotParaHeadDataSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	/// Universal aliases
	pub UniversalAliases: BTreeSet<(Location, Junction)> = BTreeSet::from_iter(
		alloc::vec![
			// Messages from Moonbeam will have Polkadot as global consensus and
			// will be put in the message queue with "Here" as origin
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
	type DestinationVersion =
		XcmVersionOfDestAndRemoteBridge<PolkadotXcm, bp_moonbeam::GlobalConsensusLocation>;

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
	type BlobDispatcher = LocalBlobDispatcher<
		MessageQueue,
		UniversalLocation,
		BridgeKusamaToPolkadotMessagesPalletInstance,
	>;
}

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking {
	use crate::bridge_config::PolkadotGlobalConsensusNetwork;
	use crate::Runtime;
	use bp_messages::source_chain::FromBridgedChainMessagesDeliveryProof;
	use bp_messages::target_chain::FromBridgedChainMessagesProof;
	use pallet_bridge_messages::LaneIdOf;
	use xcm::latest::{InteriorLocation, Location, NetworkId};
	use xcm::prelude::{GlobalConsensus, Parachain};

	/// Proof of messages, coming from Moonbeam.
	pub type FromMoonbeamMessagesProof<MI> =
		FromBridgedChainMessagesProof<bp_moonriver::Hash, LaneIdOf<Runtime, MI>>;
	/// Messages delivery proof for Moonbeam -> Moonriver.
	pub type ToMoonbeamMessagesDeliveryProof<MI> =
		FromBridgedChainMessagesDeliveryProof<bp_moonriver::Hash, LaneIdOf<Runtime, MI>>;

	pub(crate) fn open_bridge_for_benchmarks<R, XBHI, C>(
		with: pallet_xcm_bridge::LaneIdOf<R, XBHI>,
		sibling_para_id: u32,
	) -> InteriorLocation
	where
		R: pallet_xcm_bridge::Config<XBHI>,
		XBHI: 'static,
		C: xcm_executor::traits::ConvertLocation<
			bp_runtime::AccountIdOf<pallet_xcm_bridge::ThisChainOf<R, XBHI>>,
		>,
	{
		use alloc::boxed::Box;
		use pallet_xcm_bridge::{Bridge, BridgeId, BridgeState};
		use sp_runtime::traits::Zero;
		use xcm::VersionedInteriorLocation;

		// insert bridge metadata
		let lane_id = with;
		let sibling_parachain = Location::new(1, [Parachain(sibling_para_id)]);
		let universal_source = [
			GlobalConsensus(NetworkId::Kusama),
			Parachain(sibling_para_id),
		]
		.into();
		let universal_destination = [
			GlobalConsensus(PolkadotGlobalConsensusNetwork::get()),
			Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID),
		]
		.into();
		let bridge_id = BridgeId::new(&universal_source, &universal_destination);

		// insert only bridge metadata, because the benchmarks create lanes
		pallet_xcm_bridge::Bridges::<R, XBHI>::insert(
			bridge_id,
			Bridge {
				bridge_origin_relative_location: Box::new(sibling_parachain.clone().into()),
				bridge_origin_universal_location: Box::new(VersionedInteriorLocation::from(
					universal_source.clone(),
				)),
				bridge_destination_universal_location: Box::new(VersionedInteriorLocation::from(
					universal_destination,
				)),
				state: BridgeState::Opened,
				bridge_owner_account: C::convert_location(&sibling_parachain)
					.expect("valid AccountId"),
				deposit: Zero::zero(),
				lane_id,
			},
		);
		pallet_xcm_bridge::LaneToBridge::<R, XBHI>::insert(lane_id, bridge_id);

		universal_source
	}
}
