// Copyright 2024 Moonbeam foundation
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

/// System account size in bytes = Pallet_Name_Hash (16) + Storage_name_hash (16) +
/// Blake2_128Concat (16) + AccountId (20) + AccountInfo (4 + 12 + AccountData (4* 16)) = 148
pub const SYSTEM_ACCOUNT_SIZE: u64 = 148;
