use crate::*;

impl<T: Config> Inspect<H160> for Pallet<T> {
	type Balance = T::NativeBalance;

	fn total_issuance() -> Self::Balance {
		T::Balances::total_issuance()
	}

	fn minimum_balance() -> Self::Balance {
		T::Balances::minimum_balance()
	}

	fn total_balance(who: &H160) -> Self::Balance {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::total_balance(cross.as_sub())
	}

	fn balance(who: &H160) -> Self::Balance {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::balance(cross.as_sub())
	}

	fn reducible_balance(
		who: &H160,
		preservation: frame_support::traits::tokens::Preservation,
		force: frame_support::traits::tokens::Fortitude,
	) -> Self::Balance {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::reducible_balance(cross.as_sub(), preservation, force)
	}

	fn can_deposit(
		who: &H160,
		amount: Self::Balance,
		provenance: frame_support::traits::tokens::Provenance,
	) -> frame_support::traits::tokens::DepositConsequence {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::can_deposit(cross.as_sub(), amount, provenance)
	}

	fn can_withdraw(
		who: &H160,
		amount: Self::Balance,
	) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::can_withdraw(cross.as_sub(), amount)
	}
}

impl<T: Config> Unbalanced<H160> for Pallet<T> {
	fn handle_dust(dust: frame_support::traits::fungible::Dust<H160, Self>) {
		// TODO  check: is_correct?
		let dust = Dust(dust.0);
		T::Balances::handle_dust(dust)
	}

	fn write_balance(
		who: &H160,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		let cross = T::CrossAccountId::from_eth(*who);
		T::Balances::write_balance(cross.as_sub(), amount)
	}

	fn set_total_issuance(amount: Self::Balance) {
		T::Balances::set_total_issuance(amount)
	}
}

impl<T: Config> Mutate<H160> for Pallet<T> {
	fn done_mint_into(who: &H160, amount: Self::Balance) {
		let cross = T::CrossAccountId::from_eth(*who);
		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: H160::zero(),
				to: *who,
				value: amount.into(),
			}
			.to_log(T::ContractAddress::get()),
		);
		T::Balances::done_mint_into(cross.as_sub(), amount)
	}

	fn done_burn_from(who: &H160, amount: Self::Balance) {
		let cross = T::CrossAccountId::from_eth(*who);
		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: *who,
				to: H160::zero(),
				value: amount.into(),
			}
			.to_log(T::ContractAddress::get()),
		);
		T::Balances::done_burn_from(cross.as_sub(), amount)
	}

	fn done_shelve(who: &H160, amount: Self::Balance) {
		let cross = T::CrossAccountId::from_eth(*who);
		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: *who,
				to: H160::zero(),
				value: amount.into(),
			}
			.to_log(T::ContractAddress::get()),
		);
		T::Balances::done_shelve(cross.as_sub(), amount)
	}

	fn done_restore(who: &H160, amount: Self::Balance) {
		let cross = T::CrossAccountId::from_eth(*who);
		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: H160::zero(),
				to: *who,
				value: amount.into(),
			}
			.to_log(T::ContractAddress::get()),
		);
		T::Balances::done_restore(cross.as_sub(), amount)
	}

	fn done_transfer(source: &H160, dest: &H160, amount: Self::Balance) {
		let cross_source = T::CrossAccountId::from_eth(*source);
		let cross_dest = T::CrossAccountId::from_eth(*dest);
		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: *source,
				to: *dest,
				value: amount.into(),
			}
			.to_log(T::ContractAddress::get()),
		);
		T::Balances::done_transfer(cross_source.as_sub(), cross_dest.as_sub(), amount)
	}
}
