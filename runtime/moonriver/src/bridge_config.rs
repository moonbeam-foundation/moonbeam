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

use crate::xcm_config::{SelfLocation, SelfReserve, UniversalLocation};
use crate::{
	moonriver_weights, xcm_config, Balances, BridgePolkadotMessages, BridgeXcmOverMoonbeam, Get,
	MessageQueue, PolkadotXcm, Runtime, RuntimeEvent, RuntimeHoldReason, ToPolkadotXcmRouter,
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
use pallet_xcm_bridge::congestion::{
	BlobDispatcherWithChannelStatus, HereOrLocalConsensusXcmChannelManager,
	UpdateBridgeStatusXcmChannelManager,
};
use parity_scale_codec::{Decode, Encode};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::Convert;
use sp_runtime::Vec;
use sp_std::vec;
use sp_weights::Weight;
use xcm::latest::{
	AssetId, InteriorLocation, Junction, Location, MaybeErrorCode, NetworkId, OriginKind,
	SendError, SendResult, SendXcm, Xcm, XcmHash,
};
use xcm::opaque::VersionedXcm;
use xcm::prelude::{
	ExpectTransactStatus, GlobalConsensus, PalletInstance, Parachain, Transact, Unlimited,
	UnpaidExecution,
};
use xcm_builder::{
	ensure_is_remote, BridgeMessage, DispatchBlob, DispatchBlobError, ParentIsPreset,
	SiblingParachainConvertsVia,
};
use xcm_executor::traits::{validate_export, ExportXcm};

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
	pub BridgeMoonriverLocation: Location = SelfLocation::get();

	/// Price of every byte of the Kusama -> Polkadot message. Can be adjusted via
	/// governance `set_storage` call.
	pub XcmMoonriverRouterByteFee: Balance = bp_moonbeam::estimate_moonbeam_to_moonriver_byte_fee();

	/// Router expects payment with this `AssetId`.
	/// (`AssetId` has to be aligned with `BridgeTable`)
	pub XcmMoonriverRouterFeeAssetId: AssetId = SelfReserve::get().into();

	pub const RelayChainHeadersToKeep: u32 = 1024;
	pub const ParachainHeadsToKeep: u32 = 64;

	pub const PolkadotBridgeParachainPalletName: &'static str = bp_polkadot::PARAS_PALLET_NAME;
	pub const MaxPolkadotParaHeadDataSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	// see the `FEE_BOOST_PER_RELAY_HEADER` constant get the meaning of this value
	pub PriorityBoostPerRelayHeader: u64 = 32_007_814_407_814;


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

/// TODO: This struct can be removed when updating to polkadot-sdk stable2503
/// Added in https://github.com/paritytech/polkadot-sdk/pull/7126
///
/// Implementation of `SendXcm` which uses the given `ExportXcm` implementation in order to forward
/// the message over a bridge.
///
/// This is only useful when the local chain has bridging capabilities.
pub struct LocalExporter<Exporter, UniversalLocation>(PhantomData<(Exporter, UniversalLocation)>);
impl<Exporter: ExportXcm, UniversalLocation: Get<InteriorLocation>> SendXcm
	for LocalExporter<Exporter, UniversalLocation>
{
	type Ticket = Exporter::Ticket;

	fn validate(
		dest: &mut Option<Location>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<Exporter::Ticket> {
		// This `clone` ensures that `dest` is not consumed in any case.
		let d = dest.clone().take().ok_or(SendError::MissingArgument)?;
		let universal_source = UniversalLocation::get();
		let devolved =
			ensure_is_remote(universal_source.clone(), d).map_err(|_| SendError::NotApplicable)?;
		let (remote_network, remote_location) = devolved;
		let xcm = msg.take().ok_or(SendError::MissingArgument)?;

		let hash =
			(Some(Location::here()), &remote_location).using_encoded(sp_io::hashing::blake2_128);
		let channel = u32::decode(&mut hash.as_ref()).unwrap_or(0);

		validate_export::<Exporter>(
			remote_network,
			channel,
			universal_source,
			remote_location,
			xcm.clone(),
		)
		.inspect_err(|err| {
			if let SendError::NotApplicable = err {
				// We need to make sure that msg is not consumed in case of `NotApplicable`.
				*msg = Some(xcm);
			}
		})
	}

	fn deliver(ticket: Exporter::Ticket) -> Result<XcmHash, SendError> {
		Exporter::deliver(ticket)
	}
}

/// Converts encoded call to the unpaid XCM `Transact`.
pub struct UpdateBridgeStatusXcmProvider;
impl Convert<Vec<u8>, Xcm<()>> for UpdateBridgeStatusXcmProvider {
	fn convert(encoded_call: Vec<u8>) -> Xcm<()> {
		Xcm(vec![
			UnpaidExecution {
				weight_limit: Unlimited,
				check_origin: None,
			},
			Transact {
				origin_kind: OriginKind::Xcm,
				call: encoded_call.into(),
				// TODO: FAIL-CI - add some test for this or remove TODO
				fallback_max_weight: Some(Weight::from_parts(200_000_000, 6144)),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
		])
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
	// TODO
	type WeightInfo = pallet_bridge_messages::weights::BridgeWeight<Runtime>;
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

	type LocalXcmChannelManager = HereOrLocalConsensusXcmChannelManager<
		bp_xcm_bridge::BridgeId,
		// handles congestion for local chain router for local bridges
		ToPolkadotXcmRouter,
		// handles congestion for other local chains with XCM using `update_bridge_status` sent to
		// the sending chain.
		UpdateBridgeStatusXcmChannelManager<
			Runtime,
			XcmOverPolkadotInstance,
			UpdateBridgeStatusXcmProvider,
			xcm_config::LocalXcmRouter,
		>,
	>;
	// Dispatching inbound messages from the bridge and managing congestion with the local
	// receiving/destination chain
	type BlobDispatcher = BlobDispatcherWithChannelStatus<
		// Dispatches received XCM messages from other bridge
		LocalBlobDispatcher<
			MessageQueue,
			UniversalLocation,
			BridgeKusamaToPolkadotMessagesPalletInstance,
		>,
		// Provides the status of the XCMP queue's outbound queue, indicating whether messages can
		// be dispatched to the sibling.
		(),
	>;

	type CongestionLimits = ();
	// TODO
	type WeightInfo = ();
}

/// XCM router instance to BridgeHub with bridging capabilities for `Kusama` global
/// consensus with dynamic fees and back-pressure.
pub type ToPolkadotXcmRouterInstance = pallet_xcm_bridge_router::Instance1;
impl pallet_xcm_bridge_router::Config<ToPolkadotXcmRouterInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DestinationVersion = PolkadotXcm;
	type MessageExporter = pallet_xcm_bridge_router::impls::ViaLocalBridgeExporter<
		Runtime,
		ToPolkadotXcmRouterInstance,
		LocalExporter<BridgeXcmOverMoonbeam, UniversalLocation>,
	>;
	// For congestion - resolves `BridgeId` using the same algorithm as `pallet_xcm_bridge`.
	type BridgeIdResolver =
		pallet_xcm_bridge_router::impls::EnsureIsRemoteBridgeIdResolver<UniversalLocation>;
	// We don't expect here `update_bridge_status` calls, but let's allow just for root (governance,
	// ...).
	type UpdateBridgeStatusOrigin = EnsureRoot<AccountId>;

	type ByteFee = XcmMoonriverRouterByteFee;
	type FeeAsset = XcmMoonriverRouterFeeAssetId;
	// TODO
	type WeightInfo = ();
}
