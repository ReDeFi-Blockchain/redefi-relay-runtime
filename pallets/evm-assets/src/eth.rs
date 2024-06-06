use evm_coder::{abi::AbiType, generate_stubgen, solidity_interface, types::Caller};
use pallet_evm::{OnMethodCall, PrecompileHandle, PrecompileResult};
use pallet_evm_coder_substrate::{
	dispatch_to_evm,
	execution::{PreDispatch, Result},
	frontier_contract,
};

use crate::*;

#[derive(ToLog)]
pub enum ERC20Events {
	Transfer {
		#[indexed]
		from: Address,
		#[indexed]
		to: Address,
		value: U256,
	},
	Approval {
		#[indexed]
		owner: Address,
		#[indexed]
		spender: Address,
		value: U256,
	},
}

frontier_contract! {
	macro_rules! FungibleAssetsHandle_result {...}
	impl<T: Config> Contract for FungibleAssetsHandle<T> {...}
}

#[solidity_interface(name = ERC20, events(ERC20Events), enum(derive(PreDispatch)), enum_attr(weight), expect_selector = 0x942e8b22)]
impl<T: Config> FungibleAssetsHandle<T> {
	fn allowance(&self, owner: Address, spender: Address) -> Result<U256> {
		self.consume_store_reads(1)?;
		Ok(<Pallet<T>>::allowance(self.asset_id(), &owner, &spender))
	}

	fn approve(&mut self, caller: Caller, spender: Address, amount: U256) -> Result<bool> {
		self.consume_store_writes(1)?;
		let amount = amount.try_into().map_err(|_| "amount overflow")?;
		<Pallet<T>>::approve(self.asset_id(), &caller, &spender, amount, true)
			.map_err(dispatch_to_evm::<T>)?;
		Ok(true)
	}

	fn balance_of(&self, owner: Address) -> Result<U256> {
		self.consume_store_reads(1)?;
		let balance = <Pallet<T>>::balance(self.asset_id(), &owner);
		Ok(balance.into())
	}

	fn decimals(&self) -> Result<u8> {
		self.consume_store_reads(1)?;
		<Pallet<T>>::decimals(self.asset_id()).map_err(dispatch_to_evm::<T>)
	}

	fn name(&self) -> Result<String> {
		self.consume_store_reads(1)?;
		<Pallet<T>>::name_of(self.asset_id()).map_err(dispatch_to_evm::<T>)
	}

	fn symbol(&self) -> Result<String> {
		self.consume_store_reads(1)?;
		<Pallet<T>>::symbol(self.asset_id()).map_err(dispatch_to_evm::<T>)
	}

	fn total_supply(&self) -> Result<U256> {
		self.consume_store_reads(1)?;
		Ok(<Pallet<T>>::total_supply(self.asset_id()).into())
	}

	fn transfer(&mut self, caller: Caller, to: Address, amount: U256) -> Result<bool> {
		self.consume_store_reads(2)?;
		self.consume_store_writes(2)?;

		let amount = amount.try_into().map_err(|_| "amount overflow")?;

		<Pallet<T>>::transfer(self.asset_id(), &caller, &to, amount)
			.map_err(dispatch_to_evm::<T>)?;

		Ok(true)
	}

	fn transfer_from(
		&mut self,
		caller: Caller,
		from: Address,
		to: Address,
		amount: U256,
	) -> Result<bool> {
		self.consume_store_reads(3)?;
		self.consume_store_writes(3)?;
		let amount = amount.try_into().map_err(|_| "amount overflow")?;

		<Pallet<T>>::transfer_from(self.asset_id(), &caller, &from, &to, amount)
			.map_err(dispatch_to_evm::<T>)?;

		Ok(true)
	}
}

#[solidity_interface(name = ERC20Burnable, is(ERC20), enum(derive(PreDispatch)), enum_attr(weight))]
impl<T: Config> FungibleAssetsHandle<T> {
	pub fn burn(&mut self, caller: Caller, value: U256) -> Result<()> {
		self.consume_store_reads(2)?;
		self.consume_store_writes(2)?;
		let value = value.try_into().map_err(|_| "value overflow")?;
		<Pallet<T>>::burn(self.asset_id(), &caller, value).map_err(dispatch_to_evm::<T>)
	}

	pub fn burn_from(&mut self, caller: Caller, account: Address, value: U256) -> Result<()> {
		self.consume_store_reads(3)?;
		self.consume_store_writes(3)?;
		let value = value.try_into().map_err(|_| "value overflow")?;
		<Pallet<T>>::spend_allowance(self.asset_id(), &account, &caller, value)
			.map_err(dispatch_to_evm::<T>)?;
		<Pallet<T>>::burn(self.asset_id(), &account, value).map_err(dispatch_to_evm::<T>)
	}
}

#[solidity_interface(name = ERC20Mintable, is(ERC20), enum(derive(PreDispatch)), enum_attr(weight))]
impl<T: Config> FungibleAssetsHandle<T> {
	pub fn mint(&mut self, caller: Caller, to: Address, amount: U256) -> Result<()> {
		self.consume_store_reads(2)?;
		self.consume_store_writes(2)?;
		<Pallet<T>>::check_owner(self.asset_id(), &caller).map_err(dispatch_to_evm::<T>)?;
		let amount = amount.try_into().map_err(|_| "value overflow")?;
		<Pallet<T>>::mint(self.asset_id(), &to, amount).map_err(dispatch_to_evm::<T>)
	}
}

#[solidity_interface(name = XcmExtensions, is(ERC20), enum(derive(PreDispatch)), enum_attr(weight))]
impl<T: Config> FungibleAssetsHandle<T>
where
	<T as frame_system::Config>::RuntimeOrigin: From<EthereumOrigin>,
{
	#[weight(<<T as pallet_xcm::Config>::WeightInfo as PalletXcmWeightInfo>::teleport_assets() + T::DbWeight::get().reads(2_u64))]
	pub fn cross_chain_transfer(
		&mut self,
		caller: Caller,
		chain_id: ChainId,
		receiver: Address,
		amount: U256,
	) -> Result<()> {
		let amount = amount.try_into().map_err(|_| "value overflow")?;
		let locator = <T as Config>::ChainLocator::get();
		let destination = *locator.get(&chain_id).ok_or("chain not found")?;
		let relay_network = T::UniversalLocation::get()
			.global_consensus()
			.map_err(|_| "unable to get global consensus")?;
		if amount > <Pallet<T>>::balance(self.asset_id(), &caller) {
			return Err(dispatch_to_evm::<T>(
				<Error<T>>::ERC20InsufficientBalance.into(),
			));
		}

		// Determining the asset location relative to the relay.
		// For relay - 0, for parachains - 1.
		// Correctness is ensured by the correct configuration of the `ChainLocator`.
		let parents = match (destination.parent_count(), destination.interior()) {
			(1, Junctions::Here) | (1, Junctions::X1(Junction::Parachain(_))) => Ok(1),
			(0, Junctions::X1(Junction::Parachain(_))) => Ok(0),
			_ => Err("unsupported location pattern"),
		}?;

		let asset = XcmAsset {
			id: Location::new(
				parents,
				Junction::AccountKey20 {
					network: Some(relay_network),
					key: <Pallet<T>>::asset_id_to_address(self.asset_id()).into(),
				},
			)
			.into(),
			fun: Fungibility::Fungible(amount),
		};

		let beneficiary = Location::new(
			0,
			Junction::AccountKey20 {
				network: Some(relay_network),
				key: receiver.into(),
			},
		);

		let fee_asset_item = 0;
		<PalletXcm<T>>::limited_teleport_assets(
			EthereumOrigin::EthereumTransaction(caller).into(),
			Box::new(destination.into()),
			Box::new(beneficiary.into()),
			Box::new(asset.into()),
			fee_asset_item,
			WeightLimit::Unlimited,
		)
		.map_err(dispatch_to_evm::<T>)
	}
}

/// Implements [`OnMethodCall`], which delegates call to [`NativeFungibleHandle`]
pub struct AdapterOnMethodCall<T: Config>(PhantomData<*const T>);
impl<T: Config> OnMethodCall<T> for AdapterOnMethodCall<T>
where
	T::AccountId: From<[u8; 32]> + AsRef<[u8; 32]>,
	<T as frame_system::Config>::RuntimeOrigin: From<EthereumOrigin>,
{
	fn is_reserved(contract: &H160) -> bool {
		<Pallet<T>>::address_to_asset_id(contract).is_some()
	}

	fn is_used(contract: &H160) -> bool {
		<Pallet<T>>::address_to_asset_id(contract)
			.map(<Pallet<T>>::asset_exists)
			.unwrap_or_default()
	}

	fn call(handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let asset_id = <Pallet<T>>::address_to_asset_id(&handle.code_address());

		let adapter_handle =
			<FungibleAssetsHandle<T>>::new_with_gas_limit(asset_id?, handle.remaining_gas());
		pallet_evm_coder_substrate::call::<_, NativeFungibleAssetsCall<T>, _, _>(
			handle,
			adapter_handle,
		)
	}

	fn get_code(contract: &H160) -> Option<Vec<u8>> {
		Self::is_used(contract).then(|| include_bytes!("./stubs/NativeFungibleAssets.raw").to_vec())
	}
}

#[solidity_interface(
	name = NativeFungibleAssets,
	is(ERC20, ERC20Burnable, ERC20Mintable, XcmExtensions),
	enum(derive(PreDispatch))
)]
impl<T: Config> FungibleAssetsHandle<T>
where
	T::AccountId: From<[u8; 32]> + AsRef<[u8; 32]>,
	<T as frame_system::Config>::RuntimeOrigin: From<EthereumOrigin>,
{
}

generate_stubgen!(gen_impl, NativeFungibleAssetsCall<()>, true);
generate_stubgen!(gen_iface, NativeFungibleAssetsCall<()>, false);
