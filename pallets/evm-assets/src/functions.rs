use crate::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn address_to_asset_id(address: &Address) -> Option<AssetId> {
		let (prefix, id) = address.as_fixed_bytes().split_at(4);
		if prefix != T::AddressPrefix::get() {
			return None;
		}
		Some(AssetId::from_be_bytes(<[u8; 16]>::try_from(id).ok()?))
	}

	pub(crate) fn asset_id_to_address(asset: &AssetId) -> H160 {
		let mut buff = [0; 20];
		buff[..4].copy_from_slice(&T::AddressPrefix::get());
		buff[4..20].copy_from_slice(&AssetId::to_be_bytes(*asset));
		H160(buff)
	}

	pub fn balance(asset: &AssetId, account: &Address) -> Balance {
		<Balances<T>>::get(asset, account)
	}

	pub fn allowance(asset: &AssetId, owner: &Address, spender: &Address) -> U256 {
		<Approvals<T>>::get((asset, owner, spender)).into()
	}

	pub fn total_supply(asset: &AssetId) -> Balance {
		Self::maybe_total_supply(asset).unwrap_or_default()
	}

	pub fn maybe_total_supply(asset: &AssetId) -> Option<Balance> {
		<Asset<T>>::get(asset).map(|a| a.supply)
	}

	pub fn asset_details(
		asset: &AssetId,
	) -> Result<AssetDetails<Balance, Address>, sp_runtime::DispatchError> {
		<Asset<T>>::get(asset).ok_or(<Error<T>>::AssetNotFound.into())
	}

	pub fn asset_exists(asset: AssetId) -> bool {
		<Asset<T>>::contains_key(asset)
	}

	pub fn decimals(asset: &AssetId) -> Result<u8, sp_runtime::DispatchError> {
		<Metadata<T>>::get(asset)
			.map(|m| m.decimals)
			.ok_or(<Error<T>>::AssetNotFound.into())
	}

	pub fn name_of(asset: &AssetId) -> Result<String, sp_runtime::DispatchError> {
		<Metadata<T>>::get(asset)
			.map(|m| String::from_utf8_lossy(&m.name[..]).into())
			.ok_or(<Error<T>>::AssetNotFound.into())
	}

	pub fn symbol(asset: &AssetId) -> Result<String, sp_runtime::DispatchError> {
		<Metadata<T>>::get(asset)
			.map(|m| String::from_utf8_lossy(&m.symbol[..]).into())
			.ok_or(<Error<T>>::AssetNotFound.into())
	}

	pub fn transfer(
		asset: &AssetId,
		from: &Address,
		to: &Address,
		amount: Balance,
	) -> DispatchResult {
		ensure!(from != &Address::zero(), <Error<T>>::ERC20InvalidSender);
		ensure!(to != &Address::zero(), <Error<T>>::ERC20InvalidReceiver);

		Self::update(asset, from, to, amount)
	}

	pub(crate) fn update(
		asset: &AssetId,
		from: &Address,
		to: &Address,
		amount: Balance,
	) -> DispatchResult {
		if from == &Address::zero() {
			let mut asset_details = Self::asset_details(asset)?;
			asset_details.supply = asset_details
				.supply
				.checked_add(amount)
				.ok_or(ArithmeticError::Overflow)?;
			<Asset<T>>::set(asset, Some(asset_details));
		} else {
			let from_balance = Self::balance(asset, from);
			ensure!(from_balance >= amount, <Error<T>>::ERC20InsufficientBalance);
			<Balances<T>>::mutate(asset, from, |balance| *balance -= amount);
		}

		if to == &Address::zero() {
			let mut asset_details = Self::asset_details(asset)?;
			asset_details.supply = asset_details
				.supply
				.checked_sub(amount)
				.ok_or(ArithmeticError::Underflow)?;
			<Asset<T>>::set(asset, Some(asset_details));
		} else {
			<Balances<T>>::mutate(asset, to, |balance| *balance += amount);
		}

		<PalletEvm<T>>::deposit_log(
			eth::ERC20Events::Transfer {
				from: *from,
				to: *to,
				value: amount.into(),
			}
			.to_log(Self::asset_id_to_address(asset)),
		);

		Ok(())
	}

	pub fn check_receiver(receiver: &Address) -> DispatchResult {
		ensure!(
			receiver != &Address::zero(),
			<Error<T>>::ERC20InvalidReceiver
		);
		Ok(())
	}

	pub fn approve(
		asset: &AssetId,
		owner: &Address,
		spender: &Address,
		amount: Balance,
		emit_event: bool,
	) -> DispatchResult {
		ensure!(owner != &Address::zero(), <Error<T>>::ERC20InvalidApprover);
		ensure!(spender != &Address::zero(), <Error<T>>::Erc20InvalidSpender);
		<Approvals<T>>::insert((asset, owner, spender), amount);

		if emit_event {
			<PalletEvm<T>>::deposit_log(
				eth::ERC20Events::Approval {
					owner: *owner,
					spender: *spender,
					value: amount.into(),
				}
				.to_log(Self::asset_id_to_address(asset)),
			);
		};

		Ok(())
	}

	pub fn spend_allowance(
		asset: &AssetId,
		owner: &Address,
		spender: &Address,
		amount: Balance,
	) -> DispatchResult {
		let current_allowance = <Approvals<T>>::get((asset, owner, spender));
		if current_allowance != Balance::MAX {
			ensure!(
				current_allowance >= amount,
				<Error<T>>::ERC20InsufficientAllowance
			);
			return Self::approve(asset, owner, spender, amount, false);
		}
		Ok(())
	}

	pub fn transfer_from(
		asset: &AssetId,
		spender: &Address,
		from: &Address,
		to: &Address,
		amount: Balance,
	) -> DispatchResult {
		Self::spend_allowance(asset, from, spender, amount)?;
		Self::transfer(asset, from, to, amount)
	}
}
