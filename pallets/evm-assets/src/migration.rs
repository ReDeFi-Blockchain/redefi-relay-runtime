use sp_io::storage::get;

use crate::*;
pub(crate) const SUDO_STORAGE_KEY: [u8; 32] =
	hex_literal::hex!("5c0d1176a568c1f92944340dbfed9e9c530ebca703c85910e7164cb7d1c9e47b");

pub(crate) fn init_assets_with<T: Config>(accounts: &[T::AccountId], owner: &T::AccountId) {
	init_red_with::<T>(accounts, owner);
	init_gbp_with::<T>(accounts, owner);
	<SupportedAssets<T>>::set(Assets::BAX | Assets::GBP);
}

pub(crate) fn init_red_with<T: Config>(accounts: &[T::AccountId], owner: &T::AccountId) {
	let red_asset = AssetDetails::<Balance, Address> {
		supply: BALANCE * NATIVE * accounts.len() as Balance,
		owner: *T::CrossAccountId::from_sub(owner.clone()).as_eth(),
	};
	<Asset<T>>::insert(RED_ID, red_asset);

	accounts
		.iter()
		.map(|acc| *T::CrossAccountId::from_sub(acc.clone()).as_eth())
		.for_each(|adr| {
			<Balances<T>>::insert(RED_ID, adr, BALANCE * NATIVE);
		});

	let red_meta = AssetMetadata::<BoundedVec<u8, T::StringLimit>> {
		name: "redefi".as_bytes().to_vec().try_into().unwrap(),
		symbol: "RED".as_bytes().to_vec().try_into().unwrap(),
		decimals: 18,
		is_frozen: false,
	};

	<Metadata<T>>::insert(RED_ID, red_meta);
}

pub(crate) fn init_gbp_with<T: Config>(accounts: &[T::AccountId], owner: &T::AccountId) {
	let gbp_asset = AssetDetails::<Balance, Address> {
		supply: BALANCE * CURRENCY * accounts.len() as Balance,
		owner: *T::CrossAccountId::from_sub(owner.clone()).as_eth(),
	};
	<Asset<T>>::insert(GBP_ID, gbp_asset);

	accounts
		.iter()
		.map(|acc| *T::CrossAccountId::from_sub(acc.clone()).as_eth())
		.for_each(|adr| {
			<Balances<T>>::insert(GBP_ID, adr, BALANCE * CURRENCY);
		});

	let gbp_meta = AssetMetadata::<BoundedVec<u8, T::StringLimit>> {
		name: "Onchain GBP".as_bytes().to_vec().try_into().unwrap(),
		symbol: "GBP".as_bytes().to_vec().try_into().unwrap(),
		decimals: 6,
		is_frozen: false,
	};

	<Metadata<T>>::insert(GBP_ID, gbp_meta);
}

pub struct InitializationWithSudoAsHolder<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for InitializationWithSudoAsHolder<T>
where
	T::AccountId: for<'a> TryFrom<&'a [u8]>,
{
	fn on_runtime_upgrade() -> Weight {
		let sudoer_raw_key = get(&SUDO_STORAGE_KEY);
		if sudoer_raw_key.is_none() {
			log::error!(
			target: LOG_TARGET,
					"Sudo key not found - migration incomplete"
				);
			return T::DbWeight::get().reads(1);
		}
		let sudoer_raw_key = sudoer_raw_key.unwrap();
		let sudoer_key = T::AccountId::try_from(sudoer_raw_key.as_ref());

		if sudoer_key.is_err() {
			log::error!(
			target: LOG_TARGET,
					"Failed to deserialize sudo key. Value: {:?}. Migration Failed",
					sudoer_raw_key
				);
			return T::DbWeight::get().reads(1);
		}

		let sudoer_key = sudoer_key.ok().unwrap();
		let accs = [sudoer_key];

		let mut supported_assets = <SupportedAssets<T>>::get();

		if !supported_assets.contains(Assets::RED) {
			init_red_with::<T>(&accs, &accs[0]);
			supported_assets |= Assets::RED;
		}

		if !supported_assets.contains(Assets::GBP) {
			init_gbp_with::<T>(&accs, &accs[0]);
			supported_assets |= Assets::GBP;
		}

		<SupportedAssets<T>>::set(supported_assets);

		T::DbWeight::get().reads_writes(4, 10)
	}
}

pub(crate) fn try_generate_genesis_from_sudo<T: Config>() -> GenesisConfig<T>
where
	T::AccountId: for<'a> TryFrom<&'a [u8]>,
{
	let sudoer_raw_key = get(&SUDO_STORAGE_KEY);
	if sudoer_raw_key.is_none() {
		log::error!(
		target: LOG_TARGET,
				"Sudo key not found - migration incomplete"
			);
	}
	let sudoer_raw_key = sudoer_raw_key.unwrap();
	let sudoer_key = T::AccountId::try_from(sudoer_raw_key.as_ref());

	if sudoer_key.is_err() {
		log::error!(
		target: LOG_TARGET,
				"Failed to deserialize sudo key. Value: {:?}. Migration Failed",
				sudoer_raw_key
			);
	}
	let sudoer_key = sudoer_key.ok().unwrap();
	GenesisConfig {
		accounts: vec![sudoer_key],
		owner: None,
	}
}
