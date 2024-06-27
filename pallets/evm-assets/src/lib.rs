#![cfg_attr(not(feature = "std"), no_std)]

extern crate xcm as staging_xcm;
use evm_coder::ToLog;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{
		fungibles::Unbalanced,
		tokens::{fungibles, DepositConsequence, Precision, Preservation, WithdrawConsequence},
		OnRuntimeUpgrade,
	},
};
pub use pallet::*;
use pallet_ethereum::Origin as EthereumOrigin;
use pallet_evm::{account::CrossAccountId, Pallet as PalletEvm};
use pallet_evm_coder_substrate::{types::String, SubstrateRecorder, WithRecorder};
use pallet_xcm::{Pallet as PalletXcm, WeightInfo as PalletXcmWeightInfo};
use sp_core::{Get, H160, U256};
use sp_runtime::{
	traits::{TryConvert, Zero},
	ArithmeticError,
};
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData, ops::Deref, prelude::*};
use staging_xcm::{
	latest::{
		AssetId as XcmAssetId, Fungibility, Junction, Junctions, MultiAsset as XcmAsset,
		MultiLocation as Location, NetworkId,
	},
	prelude::WeightLimit,
};
pub mod types;
use types::*;

pub mod functions;

pub mod eth;

pub mod hanlde;
use hanlde::*;

mod impl_fungibles;
pub mod migration;
pub mod xcm;
pub(crate) const LOG_TARGET: &str = "runtime::evm-assets";
#[frame_support::pallet]
pub mod pallet {
	use frame_support::Blake2_128Concat;
	use migration::try_generate_genesis_from_sudo;

	use self::migration::init_assets_with;
	use super::*;

	/// The in-code storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::error]
	pub enum Error<T> {
		ERC20InsufficientAllowance,
		ERC20InvalidReceiver,
		ERC20InvalidApprover,
		ERC20InvalidSender,
		Erc20InvalidSpender,
		ERC20InsufficientBalance,
		OwnableUnauthorizedAccount,
		UnauthorizedAccount,
		AssetNotFound,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_evm_coder_substrate::Config
		+ pallet_xcm::Config
		+ pallet_ethereum::Config
	{
		/// Address prefix for assets evm mirrors
		#[pallet::constant]
		type AddressPrefix: Get<[u8; 4]>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The type must contain only correct
		/// and supported 'Location', since it is used "as is"
		/// and its use does not imply deep checks
		#[pallet::constant]
		type ChainLocator: Get<BTreeMap<ChainId, Location>>;
	}

	#[pallet::storage]
	/// Details of an asset.
	pub(super) type Asset<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, AssetDetails<Balance, Address>>;

	#[pallet::storage]
	pub(super) type Approvals<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, AssetId>,
			NMapKey<Blake2_128Concat, Address>, // owner
			NMapKey<Blake2_128Concat, Address>, // spender
		),
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type Permissions<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AssetId,
		Twox64Concat,
		Address,
		AccountPermissions,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Balances
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetId,
		Blake2_128Concat,
		Address,
		Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Metadata of an asset.
	pub(super) type Metadata<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, AssetMetadata<BoundedVec<u8, T::StringLimit>>>;

	#[pallet::storage]
	pub(super) type SupportedAssets<T: Config> =
		StorageValue<Value = Assets, QueryKind = ValueQuery>;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub accounts: Vec<T::AccountId>,
		pub owner: Option<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T>
	where
		T::AccountId: for<'a> TryFrom<&'a [u8]>,
	{
		fn build(&self) {
			if !self.accounts.is_empty() {
				init_assets_with::<T>(
					&self.accounts[..],
					self.owner.as_ref().unwrap_or(&self.accounts[0]),
				);
				return;
			};

			let sudo_config = try_generate_genesis_from_sudo::<T>();
			init_assets_with::<T>(&sudo_config.accounts, &sudo_config.accounts[0]);
		}
	}
}
