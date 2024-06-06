// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Various basic types for use in the assets pallet.

use frame_support::pallet_prelude::*;

use super::*;

pub(super) type AssetId = u128;
pub(super) type Balance = u128;
pub(super) type Address = H160;
pub(crate) type ChainId = u64;

pub(crate) const CURRENCY: Balance = 1_000_000;
pub(crate) const NATIVE: Balance = 1_000_000_000_000_000_000;
pub(crate) const BALANCE: Balance = 10_000;

pub(crate) const GBP_ID: AssetId = 0xBABB0000_00000000_00000000_00000010;
pub(crate) const RED_ID: AssetId = 0xBABB0000_00000000_00000000_00000000;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub struct AssetDetails<Balance, Address> {
	pub(super) owner: Address,
	/// The total supply across all accounts.
	pub(super) supply: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetMetadata<BoundedString> {
	/// The user friendly name of this asset. Limited in length by `StringLimit`.
	pub(super) name: BoundedString,
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub(super) symbol: BoundedString,
	/// The number of decimals this asset uses to represent one unit.
	pub(super) decimals: u8,
	/// Whether the asset metadata may be changed by a non Force origin.
	pub(super) is_frozen: bool,
}

bitflags::bitflags! {
	/// Supported assets.
	#[derive(Encode, Decode, MaxEncodedLen, Default, TypeInfo)]
	pub struct Assets: u8 {
		const BAX = 1;
		const RED = 1 << 1;
		const GBP = 1 << 2;
		const EUR = 1 << 3;
	}
}

bitflags::bitflags! {
	/// Supported assets.
	#[derive(Encode, Decode, MaxEncodedLen, Default, TypeInfo)]
	pub struct AdmninistratorPermissions: u8 {
	// TODO
	}
}
