use evm_coder::{abi::AbiType, generate_stubgen, solidity_interface, types::*};
use pallet_balances::WeightInfo;
use pallet_evm_coder_substrate::{
	call, dispatch_to_evm,
	execution::{PreDispatch, Result},
	frontier_contract, WithRecorder,
};
use sp_core::{Get, U256};

use crate::{Config, NativeFungibleHandle, Pallet, SelfWeightOf};

frontier_contract! {
	macro_rules! NativeFungibleHandle_result {...}
	impl<T: Config> Contract for NativeFungibleHandle<T> {...}
}

#[solidity_interface(name = ERC20, enum(derive(PreDispatch)), enum_attr(weight), expect_selector = 0x942e8b22)]
impl<T: Config> NativeFungibleHandle<T> {
	fn allowance(&self, _owner: Address, _spender: Address) -> Result<U256> {
		Ok(U256::zero())
	}

	fn approve(&mut self, _caller: Caller, _spender: Address, _amount: U256) -> Result<bool> {
		Err("approve not supported".into())
	}

	fn balance_of(&self, owner: Address) -> Result<U256> {
		self.consume_store_reads(1)?;
		let owner = T::CrossAccountId::from_eth(owner);
		let balance = <Pallet<T>>::balance_of(&owner);
		Ok(balance.into())
	}

	fn decimals(&self) -> Result<u8> {
		Ok(T::Decimals::get())
	}

	fn name(&self) -> Result<String> {
		Ok(T::Name::get())
	}

	fn symbol(&self) -> Result<String> {
		Ok(T::Symbol::get())
	}

	fn total_supply(&self) -> Result<U256> {
		self.consume_store_reads(1)?;
		Ok(<Pallet<T>>::total_issuance().into())
	}

	#[weight(<SelfWeightOf<T>>::transfer_allow_death())]
	fn transfer(&mut self, caller: Caller, to: Address, amount: U256) -> Result<bool> {
		let caller = T::CrossAccountId::from_eth(caller);
		let to = T::CrossAccountId::from_eth(to);
		let amount = amount.try_into().map_err(|_| "amount overflow")?;

		<Pallet<T>>::transfer(&caller, &to, amount).map_err(|e| dispatch_to_evm::<T>(e.error))?;
		Ok(true)
	}

	#[weight(<SelfWeightOf<T>>::transfer_allow_death())]
	fn transfer_from(
		&mut self,
		caller: Caller,
		from: Address,
		to: Address,
		amount: U256,
	) -> Result<bool> {
		let caller = T::CrossAccountId::from_eth(caller);
		let from = T::CrossAccountId::from_eth(from);
		let to = T::CrossAccountId::from_eth(to);
		let amount = amount.try_into().map_err(|_| "amount overflow")?;
		let budget = self
			.recorder()
			.weight_calls_budget(<StructureWeight<T>>::find_parent());

		<Pallet<T>>::transfer_from(&caller, &from, &to, amount, &budget)
			.map_err(|e| dispatch_to_evm::<T>(e.error))?;
		Ok(true)
	}
}

#[solidity_interface(
	name = UniqueNativeFungible,
	is(ERC20),
	enum(derive(PreDispatch))
)]
impl<T: Config> NativeFungibleHandle<T> where T::AccountId: From<[u8; 32]> + AsRef<[u8; 32]> {}

generate_stubgen!(gen_impl, UniqueNativeFungibleCall<()>, true);
generate_stubgen!(gen_iface, UniqueNativeFungibleCall<()>, false);

impl<T: Config> CommonEvmHandler for NativeFungibleHandle<T>
where
	T::AccountId: From<[u8; 32]> + AsRef<[u8; 32]>,
{
	const CODE: &'static [u8] = include_bytes!("./stubs/UniqueNativeFungible.raw");

	fn call(self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		call::<T, UniqueNativeFungibleCall<T>, _, _>(handle, self)
	}
}
