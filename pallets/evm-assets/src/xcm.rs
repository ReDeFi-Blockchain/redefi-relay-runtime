use staging_xcm::latest::MultiAsset as Asset;
use xcm_executor::traits::{prelude::Error as XcmError, MatchesFungibles};

use crate::*;

impl<T: Config> MatchesFungibles<AssetId, Balance> for Pallet<T> {
	fn matches_fungibles(a: &Asset) -> core::result::Result<(AssetId, Balance), XcmError> {
		let (relay_parent, relay_network_id) = match T::UniversalLocation::get() {
			Junctions::X1(Junction::GlobalConsensus(network_id)) => Ok((0, network_id)),
			Junctions::X2(Junction::GlobalConsensus(network_id), Junction::Parachain(_)) => {
				Ok((1, network_id))
			}
			_ => Err(XcmError::AssetNotHandled),
		}?;
		let XcmAssetId::Concrete(Location {
			parents,
			interior: asset_interior,
		}) = &a.id
		else {
			return Err(XcmError::AssetNotHandled);
		};

		if *parents != relay_parent {
			return Err(XcmError::AssetNotHandled);
		}

		let Junctions::X1(asset_junctions) = asset_interior else {
			return Err(XcmError::AssetNotHandled);
		};

		let Junction::AccountKey20 {
			network: Some(network_id),
			key: contract_addr,
		} = asset_junctions
		else {
			return Err(XcmError::AssetNotHandled);
		};

		if *network_id != relay_network_id {
			return Err(XcmError::AssetNotHandled);
		}

		let contract_addr = Address::from_slice(contract_addr);
		let asset = Self::address_to_asset_id(&contract_addr).ok_or(XcmError::AssetNotHandled)?;

		if Self::asset_exists(asset) {
			let Fungibility::Fungible(amount) = a.fun else {
				return Err(XcmError::AmountToBalanceConversionFailed);
			};
			return Ok((asset, amount));
		}

		Err(XcmError::AssetNotHandled)
	}
}

pub struct EthereumOriginToLocation<RuntimeOrigin, Network>(PhantomData<(RuntimeOrigin, Network)>)
where
	RuntimeOrigin: Into<Result<EthereumOrigin, RuntimeOrigin>>,
	Network: Get<Option<NetworkId>>;

impl<RuntimeOrigin, Network> TryConvert<RuntimeOrigin, Location>
	for EthereumOriginToLocation<RuntimeOrigin, Network>
where
	RuntimeOrigin: Into<Result<EthereumOrigin, RuntimeOrigin>>,
	Network: Get<Option<NetworkId>>,
{
	fn try_convert(o: RuntimeOrigin) -> Result<Location, RuntimeOrigin> {
		o.into().map(|eo| match eo {
			EthereumOrigin::EthereumTransaction(address) => Junction::AccountKey20 {
				network: Network::get(),
				key: address.into(),
			}
			.into(),
		})
	}
}
