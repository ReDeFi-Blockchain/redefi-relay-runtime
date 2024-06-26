use core::marker::PhantomData;

use frame_support::{
	parameter_types,
	traits::FindAuthor,
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	ConsensusEngineId,
};
use pallet_ethereum::PostLogContent;
use pallet_evm::{EVMCurrencyAdapter, EnsureAddressTruncated, HashedAddressMapping};
use polkadot_runtime_constants::{system_parachain::RED_ID, TOKEN_SYMBOL};
use sp_runtime::{traits::ConstU32, Perbill, RuntimeAppPublic};

use crate::*;
pub mod self_contained_call;

mod sponsoring;
use sponsoring::EthCrossChainTransferSponsorshipHandler;

pub type CrossAccountId = pallet_evm::account::BasicCrossAccountId<Runtime>;

// Assuming PoV size per read is 96 bytes: 16 for twox128(Evm), 16 for twox128(Storage), 32 for storage key, and 32 for storage value
const EVM_SLOAD_PROOF_SIZE: u64 = 96;

// ~~Assuming slowest ethereum opcode is SSTORE, with gas price of 20000 as our worst case~~
// ~~(contract, which only writes a lot of data),~~
// ~~approximating on top of our real store write weight~~
//
// The above approach is very wrong, and the reason is described there:
// https://forum.polkadot.network/t/frontier-support-for-evm-weight-v2/2470/5#problem-2
parameter_types! {
	pub const ReadsPerSecond: u64 = WEIGHT_REF_TIME_PER_SECOND / <Runtime as frame_system::Config>::DbWeight::get().read;
	pub const GasPerSecond: u64 = ReadsPerSecond::get() * 2100;
	pub const WeightTimePerGas: u64 = WEIGHT_REF_TIME_PER_SECOND / GasPerSecond::get();

	pub const BytesReadPerSecond: u64 = ReadsPerSecond::get() * EVM_SLOAD_PROOF_SIZE;
	pub const ProofSizePerGas: u64 = 0; //WEIGHT_REF_TIME_PER_SECOND / GasPerSecond::get();

	pub const WeightPerGas: Weight = Weight::from_parts(WeightTimePerGas::get(), ProofSizePerGas::get());
}

/// Limiting EVM execution to 50% of block for substrate users and management tasks
/// EVM transaction consumes more weight than substrate's, so we can't rely on them being
/// scheduled fairly
const EVM_DISPATCH_RATIO: Perbill = Perbill::from_percent(50);
parameter_types! {
	pub BlockGasLimit: U256 = U256::from((NORMAL_DISPATCH_RATIO * EVM_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WeightTimePerGas::get()).ref_time());
}

pub struct EthereumFindAuthor<F>(core::marker::PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = &Babe::authorities()[author_index as usize];
			return Some(H160::from_slice(&authority_id.0.to_raw_vec()[4..24]));
		}
		None
	}
}
const MIN_GAS_PRICE: u64 = 238_095_238_096;
pub struct FeeCalculator<T>(PhantomData<T>);
impl<T: pallet_evm::Config> fp_evm::FeeCalculator for FeeCalculator<T> {
	fn min_gas_price() -> (U256, Weight) {
		(MIN_GAS_PRICE.into(), T::DbWeight::get().reads(1))
	}
}

pub type DealWithFees = Treasury;

use fp_evm::WithdrawReason;
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use pallet_evm::OnChargeEVMTransaction;
use sp_runtime::traits::UniqueSaturatedInto;

impl pallet_evm::Config for Runtime {
	type CrossAccountId = CrossAccountId;
	type AddressMapping = HashedAddressMapping<Self::Hashing>;
	type BackwardsAddressMapping = HashedAddressMapping<Self::Hashing>;
	type BlockGasLimit = BlockGasLimit;
	type FeeCalculator = FeeCalculator<Self>;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated<Self>;
	type WithdrawOrigin = EnsureAddressTruncated<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type OnMethodCall = (
		pallet_balances_adapter::eth::AdapterOnMethodCall<Self>,
		pallet_evm_assets::eth::AdapterOnMethodCall<Self>,
	);
	type OnCreate = ();
	type ChainId = ChainId;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction =
		pallet_evm_transaction_payment::WrappedEVMCurrencyAdapter<Balances, DealWithFees>;
	type FindAuthor = EthereumFindAuthor<Babe>;
	type Timestamp = crate::Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
	type GasLimitPovSizeRatio = ProofSizePerGas;
	type OnCheckEvmTransaction = pallet_evm_transaction_payment::TransactionValidity<Self>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	// Space for revert reason. Ethereum transactions are not cheap, and overall size is much less
	// than the substrate tx size, so we can afford this
	type ExtraDataLength = ConstU32<32>;
}

impl pallet_evm_coder_substrate::Config for Runtime {}

impl pallet_evm_transaction_payment::Config for Runtime {
	type EvmSponsorshipHandler = EthCrossChainTransferSponsorshipHandler<Self>;
}

parameter_types! {
	pub const Decimals: u8 = 18;
	pub Name: String = "ReDeFi BAX".to_string();
	pub Symbol: String = TOKEN_SYMBOL.to_string();
	pub const AdapterContractAddress: H160 = H160([
		0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xBA, 0xBB,
	]);
}
#[cfg(not(feature = "testnet-id"))]
pub const RED_CHAIN_ID: u64 = 1899;

#[cfg(feature = "testnet-id")]
pub const RED_CHAIN_ID: u64 = 11899;

parameter_types! {
	pub ChainLocator: BTreeMap<u64, Location> = BTreeMap::from([(RED_CHAIN_ID, Junction::Parachain(RED_ID).into_location())]);
}

impl pallet_balances_adapter::Config for Runtime {
	type Balances = Balances;
	type NativeBalance = Balance;
	type ContractAddress = AdapterContractAddress;
	type Decimals = Decimals;
	type Name = Name;
	type Symbol = Symbol;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Self>;
	type ChainLocator = ChainLocator;
}

parameter_types! {
	pub Prefix: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
	pub StringLimit: u32 = 32;
}

impl pallet_evm_assets::Config for Runtime {
	type AddressPrefix = Prefix;
	type StringLimit = StringLimit;
	type ChainLocator = ChainLocator;
}
