use core::marker::PhantomData;

use pallet_evm::account::CrossAccountId;
use pallet_evm_transaction_payment::CallContext;
use up_sponsorship::SponsorshipHandler;

// TODO: Move addresses to config/ethereum.rs.
const BAX: [u8; 20] = hex_literal::hex!("FFFFFFFF0000000000000000000000000000BABB");
const RED: [u8; 20] = hex_literal::hex!("FFFFFFFFBABB0000000000000000000000000000");
const GBP: [u8; 20] = hex_literal::hex!("FFFFFFFFBABB0000000000000000000000000010");

// Selector for crossChainTransfer(uint64 chainId, address receiver, uint256 amount)
const CROSS_CHAIN_TRANSFER: [u8; 4] = hex_literal::hex!("EE18D38E");

pub struct EthCrossChainTransferSponsorshipHandler<T>(PhantomData<T>);

impl<T> SponsorshipHandler<T::CrossAccountId, CallContext>
	for EthCrossChainTransferSponsorshipHandler<T>
where
	T: frame_system::Config + pallet_evm::Config,
	T::AccountId: From<sp_runtime::AccountId32>,
{
	fn get_sponsor(_who: &T::CrossAccountId, call: &CallContext) -> Option<T::CrossAccountId> {
		let use_treasury = [&BAX, &RED, &GBP].contains(&call.contract_address.as_fixed_bytes())
			&& call.input.starts_with(&CROSS_CHAIN_TRANSFER);

		use_treasury.then(|| T::CrossAccountId::from_sub(crate::Treasury::account_id().into()))
	}
}
