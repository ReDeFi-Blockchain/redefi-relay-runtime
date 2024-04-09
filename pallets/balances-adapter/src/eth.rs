use evm_coder::{abi::AbiType, generate_stubgen, solidity_interface};
use pallet_evm::{OnMethodCall, PrecompileHandle, PrecompileResult};
use pallet_evm_coder_substrate::{
	dispatch_to_evm,
	execution::{PreDispatch, Result},
	frontier_contract,
};

use super::*;

frontier_contract! {
	macro_rules! NativeFungibleHandle_result {...}
	impl<T: Config> Contract for NativeFungibleHandle<T> {...}
}

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

#[solidity_interface(name = ERC20, events(ERC20Events), enum(derive(PreDispatch)), enum_attr(weight), expect_selector = 0x942e8b22)]
impl<T: Config> NativeFungibleHandle<T> {
	fn allowance(&self, owner: Address, spender: Address) -> Result<U256> {
		self.consume_store_reads(1)?;
		let owner = T::CrossAccountId::from_eth(owner);
		let spender = T::CrossAccountId::from_eth(spender);
		Ok(<Pallet<T>>::allowance(&owner, &spender))
	}

	fn approve(&mut self, caller: Caller, spender: Address, amount: U256) -> Result<bool> {
		self.consume_store_writes(1)?;
		let owner = T::CrossAccountId::from_eth(caller);
		let spender = T::CrossAccountId::from_eth(spender);
		let amount = amount.try_into().map_err(|_| "amount overflow")?;
		<Pallet<T>>::approve(&owner, &spender, amount, true).map_err(dispatch_to_evm::<T>)?;
		Ok(true)
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

		<Pallet<T>>::transfer(&caller, &to, amount).map_err(dispatch_to_evm::<T>)?;
		Ok(true)
	}

	#[weight(<SelfWeightOf<T>>::transfer_allow_death() + T::DbWeight::get().writes(1_u64))]
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

		<Pallet<T>>::transfer_from(&caller, &from, &to, amount).map_err(dispatch_to_evm::<T>)?;
		Ok(true)
	}
}

/// Implements [`OnMethodCall`], which delegates call to [`NativeFungibleHandle`]
pub struct AdapterOnMethodCall<T: Config>(PhantomData<*const T>);
impl<T: Config> OnMethodCall<T> for AdapterOnMethodCall<T>
where
	T::AccountId: AsRef<[u8; 32]>,
{
	fn is_reserved(contract: &sp_core::H160) -> bool {
		contract == &T::ContractAddress::get()
	}

	fn is_used(contract: &sp_core::H160) -> bool {
		contract == &T::ContractAddress::get()
	}

	fn call(handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		if handle.code_address() != T::ContractAddress::get() {
			return None;
		}

		let adapter_handle = <NativeFungibleHandle<T>>::new_with_gas_limit(handle.remaining_gas());
		pallet_evm_coder_substrate::call(handle, adapter_handle)
	}

	fn get_code(contract: &sp_core::H160) -> Option<Vec<u8>> {
		(contract == &T::ContractAddress::get())
			.then(|| include_bytes!("./stubs/NativeFungible.raw").to_vec())
	}
}

#[solidity_interface(
	name = NativeFungible,
	is(ERC20),
	enum(derive(PreDispatch))
)]
impl<T: Config> NativeFungibleHandle<T> where T::AccountId: From<[u8; 32]> + AsRef<[u8; 32]> {}

generate_stubgen!(gen_impl, NativeFungibleCall<()>, true);
generate_stubgen!(gen_iface, NativeFungibleCall<()>, false);
