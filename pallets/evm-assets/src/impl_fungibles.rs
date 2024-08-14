use crate::*;

impl<T: Config> fungibles::Inspect<Address> for Pallet<T> {
	type AssetId = AssetId;

	type Balance = Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		<Asset<T>>::get(asset)
			.map(|a| a.supply)
			.unwrap_or_else(Zero::zero)
	}

	fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
		Zero::zero()
	}

	fn total_balance(asset: Self::AssetId, who: &Address) -> Self::Balance {
		Self::balance(&asset, who)
	}

	fn balance(asset: Self::AssetId, who: &Address) -> Self::Balance {
		Self::balance(&asset, who)
	}

	fn reducible_balance(
		asset: Self::AssetId,
		who: &Address,
		_preservation: frame_support::traits::tokens::Preservation,
		_force: frame_support::traits::tokens::Fortitude,
	) -> Self::Balance {
		Self::balance(&asset, who)
	}

	fn can_deposit(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
		_provenance: frame_support::traits::tokens::Provenance,
	) -> frame_support::traits::tokens::DepositConsequence {
		Self::balance(&asset, who)
			.checked_add(amount)
			.map(|_| DepositConsequence::Success)
			.unwrap_or(DepositConsequence::Overflow)
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
	) -> frame_support::traits::tokens::WithdrawConsequence<Self::Balance> {
		Self::balance(&asset, who)
			.checked_sub(amount)
			.map(|_| WithdrawConsequence::Success)
			.unwrap_or(WithdrawConsequence::Underflow)
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		Self::asset_exists(asset)
	}
}

impl<T: Config> fungibles::Unbalanced<Address> for Pallet<T> {
	fn handle_dust(_dust: fungibles::Dust<Address, Self>) {}

	fn write_balance(
		_asset: Self::AssetId,
		_who: &Address,
		_amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		Err(DispatchError::Unavailable)
	}

	fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) {
		<Asset<T>>::mutate_exists(asset, |a| {
			if let Some(asset_details) = a {
				asset_details.supply = amount
			}
		})
	}

	fn decrease_balance(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
		/*	Ignore these arguments since ERC20\EVM does not imply dust removal
		or the privilege with which the withdrawal operation is conducted.
		Note: This implementation is aimed at providing
		the functionality of `TransactAsset` adapters.	*/
		_precision: frame_support::traits::tokens::Precision,
		_preservation: frame_support::traits::tokens::Preservation,
		_force: frame_support::traits::tokens::Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		Self::burn(&asset, who, amount).map(|_| amount)
	}

	fn increase_balance(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
		// see decrease_balance comment
		_precision: frame_support::traits::tokens::Precision,
	) -> Result<Self::Balance, DispatchError> {
		Self::mint(&asset, who, amount).map(|_| amount)
	}
}

impl<T: Config> fungibles::Mutate<Address> for Pallet<T> {
	fn mint_into(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Self::increase_balance(asset, who, amount, Precision::Exact)
	}

	fn burn_from(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
		preservation: Preservation,
		precision: frame_support::traits::tokens::Precision,
		force: frame_support::traits::tokens::Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		Self::decrease_balance(
			asset,
			who,
			amount,
			precision,
			/* No dust support - account should live */
			preservation,
			force,
		)
	}

	fn shelve(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Self::burn(&asset, who, amount).map(|_| amount)
	}

	fn restore(
		asset: Self::AssetId,
		who: &Address,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Self::mint(&asset, who, amount).map(|_| amount)
	}

	fn transfer(
		asset: Self::AssetId,
		source: &Address,
		dest: &Address,
		amount: Self::Balance,
		// see decrease_balance comment
		_preservation: frame_support::traits::tokens::Preservation,
	) -> Result<Self::Balance, DispatchError> {
		Self::transfer(&asset, source, dest, amount).map(|_| amount)
	}
}
