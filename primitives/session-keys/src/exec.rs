// Copyright 2019-2022 PureStake Inc.
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

//! Client code to verify the VRF digest when executing the block
use crate::digest::PreDigest;
use crate::vrf::{make_transcript, GetVrfInput, VrfId, VrfInput, VRF_ENGINE_ID};
use crate::KeysLookup;
use frame_support::traits::ExecuteBlock;
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Decode;
use sp_api::{BlockT, HeaderT};
use sp_application_crypto::ByteArray;
use sp_consensus_babe::Slot;
use sp_consensus_vrf::schnorrkel;
use sp_core::H256;

pub struct BlockExecutor<B, K, I>(sp_std::marker::PhantomData<(B, K, I)>);

impl<Block, B, K, I> ExecuteBlock<Block> for BlockExecutor<B, K, I>
where
	Block: BlockT,
	B: ExecuteBlock<Block>,
	K: KeysLookup<NimbusId, VrfId>,
	I: GetVrfInput<VrfInput<Slot, H256>>,
{
	fn execute_block(block: Block) {
		let (header, extrinsics) = block.deconstruct();

		let mut block_author_vrf_id: Option<VrfId> = None;
		let PreDigest {
			vrf_output,
			vrf_proof,
		} = header
			.digest()
			.logs
			.iter()
			.filter_map(|s| s.as_pre_runtime())
			.filter_map(|(id, mut data)| {
				if id == VRF_ENGINE_ID {
					PreDigest::decode(&mut data).ok()
				} else {
					if id == NIMBUS_ENGINE_ID {
						let nimbus_id = NimbusId::decode(&mut data)
							.expect("NimbusId encoded in pre-runtime digest must be valid");

						block_author_vrf_id = Some(
							K::lookup_keys(&nimbus_id).expect("No VRF Key mapped to this NimbusId"),
						);
					}
					None
				}
			})
			.next()
			.expect("VRF PreDigest must be decoded from the digests");
		let block_author_vrf_id =
			block_author_vrf_id.expect("VrfId encoded in pre-runtime digest must be valid");
		let pubkey = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
			.expect("Expect VrfId to be valid schnorrkel public key");
		let VrfInput {
			slot_number,
			storage_root,
		} = I::get_vrf_input();
		let transcript = make_transcript::<H256>(slot_number, storage_root);
		if pubkey
			.vrf_verify(transcript, &vrf_output, &vrf_proof)
			.is_err()
		{
			panic!("VRF verification failed");
		}
		// pass to inner executor
		B::execute_block(Block::new(header, extrinsics));
	}
}
