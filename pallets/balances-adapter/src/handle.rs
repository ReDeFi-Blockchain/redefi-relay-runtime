use crate::*;

/// Handle for native fungible collection
pub struct NativeFungibleHandle<T: Config>(SubstrateRecorder<T>);

impl<T: Config> NativeFungibleHandle<T> {
	/// Creates a handle
	pub fn new() -> NativeFungibleHandle<T> {
		Self(SubstrateRecorder::new(u64::MAX))
	}

	/// Creates a handle
	pub fn new_with_gas_limit(gas_limit: u64) -> NativeFungibleHandle<T> {
		Self(SubstrateRecorder::new(gas_limit))
	}

	/// Check if the collection is internal
	pub fn check_is_internal(&self) -> DispatchResult {
		Ok(())
	}
}

impl<T: Config> Default for NativeFungibleHandle<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> WithRecorder<T> for NativeFungibleHandle<T> {
	fn recorder(&self) -> &pallet_evm_coder_substrate::SubstrateRecorder<T> {
		&self.0
	}
	fn into_recorder(self) -> pallet_evm_coder_substrate::SubstrateRecorder<T> {
		self.0
	}
}

impl<T: Config> Deref for NativeFungibleHandle<T> {
	type Target = SubstrateRecorder<T>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
