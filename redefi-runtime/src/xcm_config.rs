// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! XCM configuration for Polkadot.

use frame_support::{
	parameter_types,
	traits::{ContainsPair, Equals, Everything, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_runtime_constants::{
	currency::CENTS,
	system_parachain::*,
	xcm::body::FELLOWSHIP_ADMIN_INDEX,
};
use polkadot_runtime_common::xcm_sender::{ChildParachainRouter, NoPriceForMessageDelivery};
use polkadot_runtime_parachains::FeeTracker;
use sp_core::ConstU32;
use xcm::latest::{prelude::*, Fungibility};
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, ChildParachainAsNative, ChildParachainConvertsVia,
	DescribeAllTerminal, DescribeFamily, FrameTransactionalProcessor,
	FungibleAdapter as XcmFungibleAdapter, HashedDescription, IsConcrete, MintLocation, OriginToPluralityVoice, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId,
	WeightInfoBounds, WithComputedOrigin, WithUniqueTopic, XcmFeeToAccount, XcmFeeManagerFromComponents,
};
use xcm_executor::{traits::WithOriginFilter, AssetsInHolding};

use self::{ethereum::AdapterContractAddress, fungible_adapter::FungibleAdapter};
use super::*;

parameter_types! {
	pub const RootLocation: Location = Here.into_location();
	/// The location of the DOT token, from the context of this chain. Since this token is native to this
	/// chain, we make it synonymous with it and thus it is the `Here` location, which means "equivalent to
	/// the context".
	pub const TokenLocation: Location = Here.into_location();
	/// The ReDeFi network ID. This is named.
	pub const ThisNetwork: NetworkId = NetworkId::Ethereum { chain_id: ChainId::get() };
	/// Our location in the universe of consensus systems.
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(ThisNetwork::get())].into();
	/// The Checking Account, which holds any native assets that have been teleported out and not back in (yet).
	pub CheckAccount: AccountId = XcmPallet::check_account();
	/// The Checking Account along with the indication that the local chain is able to mint tokens.
	pub LocalCheckAccount: (AccountId, MintLocation) = (CheckAccount::get(), MintLocation::Local);
	/// Account of the treasury pallet.
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub NativeAssetXcmEvmLocation: Location = Location::new(0, Junction::AccountKey20 { network: Some(ThisNetwork::get()), key: AdapterContractAddress::get().into() });
}

/// The canonical means of converting a `MultiLocation` into an `AccountId`, used when we want to
/// determine the sovereign account controlled by a location.
pub type SovereignAccountOf = (
	// We can convert a child parachain using the standard `AccountId` conversion.
	ChildParachainConvertsVia<ParaId, AccountId>,
	// We can directly alias an `AccountId32` into a local account.
	AccountId32Aliases<ThisNetwork, AccountId>,
	// We map evm locations to Substrate mirror
	CrossAccountLocationMapperToSubstrate<AccountKey20Aliases<ThisNetwork, H160>, Runtime>,
	// Foreign locations alias into accounts according to a hash of their standard description.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

pub struct CrossAccountLocationMapperToEth<LocationConverter, Runtime>(
	PhantomData<(LocationConverter, Runtime)>,
)
where
	Runtime: pallet_evm::Config,
	LocationConverter: ConvertLocation<<Runtime as frame_system::Config>::AccountId>;

impl<Runtime, LocationConverter> ConvertLocation<H160>
	for CrossAccountLocationMapperToEth<LocationConverter, Runtime>
where
	Runtime: pallet_evm::Config,
	LocationConverter: ConvertLocation<<Runtime as frame_system::Config>::AccountId>,
{
	fn convert_location(location: &Location) -> Option<H160> {
		LocationConverter::convert_location(location).map(|account| {
			*<Runtime as pallet_evm::Config>::CrossAccountId::from_sub(account).as_eth()
		})
	}
}

pub struct CrossAccountLocationMapperToSubstrate<LocationConverter, Runtime>(
	PhantomData<(LocationConverter, Runtime)>,
)
where
	Runtime: pallet_evm::Config,
	LocationConverter: ConvertLocation<H160>;

impl<Runtime, LocationConverter> ConvertLocation<Runtime::AccountId>
	for CrossAccountLocationMapperToSubstrate<LocationConverter, Runtime>
where
	Runtime: pallet_evm::Config,
	LocationConverter: ConvertLocation<H160>,
{
	fn convert_location(location: &Location) -> Option<Runtime::AccountId> {
		LocationConverter::convert_location(location).map(|account| {
			<Runtime as pallet_evm::Config>::CrossAccountId::from_eth(account)
				.as_sub()
				.clone()
		})
	}
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type EvmAssetsLocationToAccountId20 = (
	// The Parachain origin converts to the `AccountId20`.
	CrossAccountLocationMapperToEth<ChildParachainConvertsVia<ParaId, AccountId>, Runtime>,
	AccountKey20Aliases<ThisNetwork, H160>,
);

pub type EvmAssetsTransactor =
	FungiblesAdapter<EvmAssets, EvmAssets, EvmAssetsLocationToAccountId20, H160, NoChecking, ()>;

/// Our asset transactor. This is what allows us to interact with the runtime assets from the point
/// of view of XCM-only concepts like `MultiLocation` and `MultiAsset`.
///
/// Ours is only aware of the Balances pallet, which is mapped to `TokenLocation`.
pub type LocalAssetTransactor = XcmFungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<TokenLocation>,
	// We can convert the MultiLocations with our converter above:
	SovereignAccountOf,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We track our teleports in/out to keep total issuance correct.
	LocalCheckAccount,
>;

pub type EvmLocalAssetTransactor = FungibleAdapter<
	// Use this currency:
	BalancesAdapter,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<NativeAssetXcmEvmLocation>,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	EvmAssetsLocationToAccountId20,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	H160,
	// We don't track any teleports.
	(),
>;

pub type AssetTransactor = (
	EvmLocalAssetTransactor,
	EvmAssetsTransactor,
	LocalAssetTransactor,
);

/// The means that we convert an XCM origin `MultiLocation` into the runtime's `Origin` type for
/// local dispatch. This is a conversion function from an `OriginKind` type along with the
/// `MultiLocation` value and returns an `Origin` value or an error.
type LocalOriginConverter = (
	// If the origin kind is `Sovereign`, then return a `Signed` origin with the account determined
	// by the `SovereignAccountOf` converter.
	SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
	// If the origin kind is `Native` and the XCM origin is a child parachain, then we can express
	// it with the special `parachains_origin::Origin` origin variant.
	ChildParachainAsNative<parachains_origin::Origin, RuntimeOrigin>,
	// If the origin kind is `Native` and the XCM origin is the `AccountId32` location, then it can
	// be expressed using the `Signed` origin variant.
	SignedAccountId32AsNative<ThisNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	/// The amount of weight an XCM operation takes. This is a safe overestimate.
	pub const BaseXcmWeight: Weight = Weight::from_parts(1_000_000_000, 1024);
	/// Maximum number of instructions in a single XCM fragment. A sanity check against weight
	/// calculations getting too crazy.
	pub const MaxInstructions: u32 = 100;
	/// The asset ID for the asset that we use to pay for message delivery fees.
	pub FeeAssetId: AssetId = AssetId(TokenLocation::get());
	/// The base fee for the message delivery fees.
	pub const BaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

pub type PriceForChildParachainDelivery = NoPriceForMessageDelivery<<Dmp as FeeTracker>::Id>;

/// The XCM router. When we want to send an XCM message, we use this type. It amalgamates all of our
/// individual routers.
pub type XcmRouter = WithUniqueTopic<(
	// Only one router so far - use DMP to communicate with child parachains.
	ChildParachainRouter<Runtime, XcmPallet, PriceForChildParachainDelivery>,
)>;

parameter_types! {
	pub const Dot: AssetFilter = Wild(AllOf { fun: WildFungible, id: AssetId(TokenLocation::get()) });
	pub AssetHubLocation: Location = Parachain(ASSET_HUB_ID).into_location();
	pub DotForAssetHub: (AssetFilter, Location) = (Dot::get(), AssetHubLocation::get());
	pub CollectivesLocation: Location = Parachain(COLLECTIVES_ID).into_location();
	pub DotForCollectives: (AssetFilter, Location) = (Dot::get(), CollectivesLocation::get());
	pub BridgeHubLocation: Location = Parachain(BRIDGE_HUB_ID).into_location();
	pub DotForBridgeHub: (AssetFilter, Location) = (Dot::get(), BridgeHubLocation::get());
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct EvmAssetsOnRelay;
impl ContainsPair<Asset, Location> for EvmAssetsOnRelay {
	fn contains(a: &Asset, b: &Location) -> bool {
		let unpack = (a.id.0.unpack(), &a.fun);
		let (
			(
				0,
				&[Junction::AccountKey20 {
					network: Some(network),
					key,
				}],
			),
			&Fungibility::Fungible(_),
		) = unpack
		else {
			return false;
		};

		if network != ThisNetwork::get() {
			return false;
		}

		if !key.starts_with(&[0xFF, 0xFF, 0xFF, 0xFF]) {
			return false;
		}

		let (
			0,
			&[Junction::Parachain(_)],
		) = b.unpack() else {
			return false;
		};
		true
	}
}

#[allow(unused_parens)]
/// ReDeFi Relay recognizes/respects EvmAssets as teleporters.
pub type TrustedTeleporters = (EvmAssetsOnRelay);

pub struct FreeForAll;

impl WeightTrader for FreeForAll {
	fn new() -> Self {
		Self
	}

	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: AssetsInHolding,
		_xcm: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		log::trace!(target: "fassets::weight", "buy_weight weight: {:?}, payment: {:?}", weight, payment);
		Ok(payment)
	}
}

pub struct OnlyParachains;
impl Contains<Location> for OnlyParachains {
	fn contains(loc: &Location) -> bool {
		matches!(loc.unpack(), (0, [Parachain(_)]))
	}
}

pub struct LocalPlurality;
impl Contains<Location> for LocalPlurality {
	fn contains(loc: &Location) -> bool {
		matches!(loc.unpack(), (0, [Plurality { .. }]))
	}
}

/// The barriers one of which must be passed for an XCM message to be executed.
pub type Barrier = TrailingSetTopicAsId<(
	// Weight that is paid for may be consumed.
	TakeWeightCredit,
	// Expected responses are OK.
	AllowKnownQueryResponses<XcmPallet>,
	WithComputedOrigin<
		(
			// If the message is one that immediately attempts to pay for execution, then allow it.
			AllowTopLevelPaidExecutionFrom<Everything>,
			// Subscriptions for version tracking are OK.
			AllowSubscriptionsFrom<OnlyParachains>,
			// Collectives and Fellows plurality get free execution.
			// AllowExplicitUnpaidExecutionFrom<CollectivesOrFellows>,
		),
		UniversalLocation,
		ConstU32<8>,
	>,
)>;

/// Locations that will not be charged fees in the executor, neither for execution nor delivery.
/// We only waive fees for system functions, which these locations represent.
pub type WaivedLocations = (SystemParachains, Equals<RootLocation>, LocalPlurality);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactor;
	type OriginConverter = LocalOriginConverter;
	// Polkadot Relay recognises no chains which act as reserves.
	type IsReserve = ();
	type IsTeleporter = TrustedTeleporters;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::PolkadotXcmWeight<RuntimeCall>,
		RuntimeCall,
		MaxInstructions,
	>;
	// The weight trader piggybacks on the existing transaction-fee conversion logic.
	type Trader = FreeForAll;
	type ResponseHandler = XcmPallet;
	type AssetTrap = XcmPallet;
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = XcmPallet;
	type SubscriptionService = XcmPallet;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = XcmFeeManagerFromComponents<
		WaivedLocations,
		XcmFeeToAccount<Self::AssetTransactor, AccountId, TreasuryAccount>,
	>;
	// No bridges yet...
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = WithOriginFilter<SafeCallFilter>;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = XcmPallet;
}

parameter_types! {
	// `GeneralAdmin` pluralistic body.
	pub const GeneralAdminBodyId: BodyId = BodyId::Administration;
	// StakingAdmin pluralistic body.
	pub const StakingAdminBodyId: BodyId = BodyId::Defense;
	// FellowshipAdmin pluralistic body.
	pub const FellowshipAdminBodyId: BodyId = BodyId::Index(FELLOWSHIP_ADMIN_INDEX);
	// `Treasurer` pluralistic body.
	pub const TreasurerBodyId: BodyId = BodyId::Treasury;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parachain(1000).into());
}

/// Type to convert the `GeneralAdmin` origin to a Plurality `MultiLocation` value.
pub type GeneralAdminToPlurality =
	OriginToPluralityVoice<RuntimeOrigin, GeneralAdmin, GeneralAdminBodyId>;

/// Type to convert an `Origin` type value into a `MultiLocation` value which represents an interior
/// location of this chain.
pub type LocalOriginToLocation = (
	GeneralAdminToPlurality,
	// And a usual Signed origin to be used in XCM as a corresponding AccountId32
	SignedToAccountId32<RuntimeOrigin, AccountId, ThisNetwork>,
	// For Evm precompiles support.
	pallet_evm_assets::xcm::EthereumOriginToLocation<RuntimeOrigin, ThisNetwork>,
);

/// Type to convert the `StakingAdmin` origin to a Plurality `MultiLocation` value.
pub type StakingAdminToPlurality =
	OriginToPluralityVoice<RuntimeOrigin, StakingAdmin, StakingAdminBodyId>;

/// Type to convert the `FellowshipAdmin` origin to a Plurality `MultiLocation` value.
pub type FellowshipAdminToPlurality =
	OriginToPluralityVoice<RuntimeOrigin, FellowshipAdmin, FellowshipAdminBodyId>;

/// Type to convert the `Treasurer` origin to a Plurality `MultiLocation` value.
pub type TreasurerToPlurality = OriginToPluralityVoice<RuntimeOrigin, Treasurer, TreasurerBodyId>;

/// Type to convert a pallet `Origin` type value into a `MultiLocation` value which represents an
/// interior location of this chain for a destination chain.
pub type LocalPalletOriginToLocation = (
	// GeneralAdmin origin to be used in XCM as a corresponding Plurality `MultiLocation` value.
	GeneralAdminToPlurality,
	// StakingAdmin origin to be used in XCM as a corresponding Plurality `MultiLocation` value.
	StakingAdminToPlurality,
	// FellowshipAdmin origin to be used in XCM as a corresponding Plurality `MultiLocation` value.
	FellowshipAdminToPlurality,
	// `Treasurer` origin to be used in XCM as a corresponding Plurality `MultiLocation` value.
	TreasurerToPlurality,
);

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// We only allow the root, the general admin, the fellowship admin and the staking admin to send
	// messages.
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalPalletOriginToLocation>;
	type XcmRouter = XcmRouter;
	// Anyone can execute XCM messages locally...
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// ...but they must match our filter, which rejects all.
	type XcmExecuteFilter = Nothing; // == Deny All
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything; // == Allow All
	type XcmReserveTransferFilter = Everything; // == Allow All
	type Weigher = WeightInfoBounds<
		crate::weights::xcm::PolkadotXcmWeight<RuntimeCall>,
		RuntimeCall,
		MaxInstructions,
	>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = SovereignAccountOf;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type WeightInfo = crate::weights::pallet_xcm::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
}
