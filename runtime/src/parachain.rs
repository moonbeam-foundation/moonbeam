// Copyright 2019-2020 PureStake Inc.
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

#[macro_export]
macro_rules! runtime_parachain {
	() => {
		/// This runtime version.
		pub const VERSION: RuntimeVersion = RuntimeVersion {
			spec_name: create_runtime_str!("moonbase-alphanet"),
			impl_name: create_runtime_str!("moonbase-alphanet"),
			authoring_version: 3,
			spec_version: 11,
			impl_version: 1,
			apis: RUNTIME_API_VERSIONS,
			transaction_version: 2,
		};



		// TODO Consensus not supported in parachain
		impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
			fn find_author<'a, I>(_digests: I) -> Option<H160>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				None
			}
		}

		pub struct PhantomAura;
		impl FindAuthor<u32> for PhantomAura {
			fn find_author<'a, I>(_digests: I) -> Option<u32>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				Some(0 as u32)
			}
		}


	};
}
