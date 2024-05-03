use crate::*;

/// Handle for asset as an ERC20 collection
pub struct FungibleAssetsHandle<T: Config> {
	asset_id: AssetId,
	recorder: SubstrateRecorder<T>,
}

impl<T: Config> FungibleAssetsHandle<T> {
	/// Creates a handle
	pub fn new(asset_id: AssetId) -> FungibleAssetsHandle<T> {
		Self::new_with_gas_limit(asset_id, u64::MAX)
	}

	/// Creates a handle
	pub fn new_with_gas_limit(asset_id: AssetId, gas_limit: u64) -> FungibleAssetsHandle<T> {
		Self {
			asset_id,
			recorder: SubstrateRecorder::new(gas_limit),
		}
	}

	/// Returns `AssetId` reference
	pub fn asset_id(&self) -> &AssetId {
		&self.asset_id
	}
}

impl<T: Config> Default for FungibleAssetsHandle<T> {
	fn default() -> Self {
		Self::new(Default::default())
	}
}

impl<T: Config> WithRecorder<T> for FungibleAssetsHandle<T> {
	fn recorder(&self) -> &pallet_evm_coder_substrate::SubstrateRecorder<T> {
		&self.recorder
	}
	fn into_recorder(self) -> pallet_evm_coder_substrate::SubstrateRecorder<T> {
		self.recorder
	}
}

impl<T: Config> Deref for FungibleAssetsHandle<T> {
	type Target = SubstrateRecorder<T>;

	fn deref(&self) -> &Self::Target {
		&self.recorder
	}
}
