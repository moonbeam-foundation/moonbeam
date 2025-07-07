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
	moonbeam_weights, Balances, BridgeKusamaMessages, BridgeXcmOverMoonriver, MessageQueue,
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
use xcm::latest::{Junction, Location};
use xcm::prelude::{GlobalConsensus, InteriorLocation, NetworkId, PalletInstance};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};

parameter_types! {
	pub BridgePolkadotToKusamaMessagesPalletInstance: InteriorLocation = [PalletInstance(<BridgeKusamaMessages as PalletInfoAccess>::index() as u8)].into();
	pub KusamaGlobalConsensusNetwork: NetworkId = NetworkId::Kusama;
	pub KusamaGlobalConsensusNetworkLocation: Location = Location::new(
		2,
		[GlobalConsensus(KusamaGlobalConsensusNetwork::get())]
	);

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const KusamaBridgeParachainPalletName: &'static str = bp_kusama::PARAS_PALLET_NAME;
	pub const MaxKusamaParaHeadDataSize: u32 = bp_kusama::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	/// Universal aliases
	pub UniversalAliases: BTreeSet<(Location, Junction)> = BTreeSet::from_iter(
		alloc::vec![
			// Messages from Moonriver will have Kusama as global consensus and
			// will be put in the message queue with "Here" as origin
			(SelfLocation::get(), GlobalConsensus(KusamaGlobalConsensusNetwork::get()))
		]
	);

	pub storage BridgeDeposit: Balance = 0;
}

impl Contains<(Location, Junction)> for UniversalAliases {
	fn contains(alias: &(Location, Junction)) -> bool {
		UniversalAliases::get().contains(alias)
	}
}

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
pub type WithKusamaMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithKusamaMessagesInstance> for Runtime {
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
	type LaneId = bp_moonbeam::LaneId;

	type DeliveryPayments = ();
	type DeliveryConfirmationPayments = (); // Only necessary if we want to reward relayers

	type MessageDispatch = BridgeXcmOverMoonriver;
	type OnMessagesDelivered = BridgeXcmOverMoonriver;
	type WeightInfo = moonbeam_weights::pallet_bridge_messages::WeightInfo<Runtime>;
}

/// Add support for the export and dispatch of XCM programs withing
/// `WithKusamaMessagesInstance`.
pub type XcmOverKusamaInstance = pallet_xcm_bridge::Instance1;
impl pallet_xcm_bridge::Config<XcmOverKusamaInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type UniversalLocation = UniversalLocation;
	type BridgedNetwork = KusamaGlobalConsensusNetworkLocation;
	type BridgeMessagesPalletInstance = WithKusamaMessagesInstance;

	type MessageExportPrice = ();
	type DestinationVersion =
		XcmVersionOfDestAndRemoteBridge<PolkadotXcm, bp_moonriver::GlobalConsensusLocation>;

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
		BridgePolkadotToKusamaMessagesPalletInstance,
	>;
}

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking {
	use crate::bridge_config::KusamaGlobalConsensusNetwork;
	use crate::Runtime;
	use bp_messages::source_chain::FromBridgedChainMessagesDeliveryProof;
	use bp_messages::target_chain::FromBridgedChainMessagesProof;
	use pallet_bridge_messages::LaneIdOf;
	use xcm::latest::{InteriorLocation, Location, NetworkId};
	use xcm::prelude::{GlobalConsensus, Parachain};

	/// Proof of messages, coming from Moonbeam.
	pub type FromMoonriverMessagesProof<MI> =
		FromBridgedChainMessagesProof<bp_moonriver::Hash, LaneIdOf<Runtime, MI>>;
	/// Messages delivery proof for Moonbeam -> Moonriver.
	pub type ToMoonriverMessagesDeliveryProof<MI> =
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
			GlobalConsensus(KusamaGlobalConsensusNetwork::get()),
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
