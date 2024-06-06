use crate::*;
/// A call filter for the XCM Transact instruction. This is a temporary measure until we
/// properly account for proof size weights.
///
/// Calls that are allowed through this filter must:
/// 1. Have a fixed weight;
/// 2. Cannot lead to another call being made;
/// 3. Have a defined proof size weight, e.g. no unbounded vecs in call parameters.
pub struct SafeCallFilter;
impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		#[cfg(feature = "runtime-benchmarks")]
		{
			if matches!(
				call,
				RuntimeCall::System(frame_system::Call::remark_with_event { .. })
			) {
				return true;
			}
		}

		match call {
			RuntimeCall::System(
				frame_system::Call::kill_prefix { .. } | frame_system::Call::set_heap_pages { .. },
			)
			| RuntimeCall::Babe(..)
			| RuntimeCall::Timestamp(..)
			| RuntimeCall::Indices(..)
			| RuntimeCall::Balances(..)
			| RuntimeCall::Crowdloan(
				crowdloan::Call::create { .. }
				| crowdloan::Call::contribute { .. }
				| crowdloan::Call::withdraw { .. }
				| crowdloan::Call::refund { .. }
				| crowdloan::Call::dissolve { .. }
				| crowdloan::Call::edit { .. }
				| crowdloan::Call::poke { .. }
				| crowdloan::Call::contribute_all { .. },
			)
			| RuntimeCall::Staking(
				pallet_staking::Call::bond { .. }
				| pallet_staking::Call::bond_extra { .. }
				| pallet_staking::Call::unbond { .. }
				| pallet_staking::Call::withdraw_unbonded { .. }
				| pallet_staking::Call::validate { .. }
				| pallet_staking::Call::nominate { .. }
				| pallet_staking::Call::chill { .. }
				| pallet_staking::Call::set_payee { .. }
				| pallet_staking::Call::set_controller { .. }
				| pallet_staking::Call::set_validator_count { .. }
				| pallet_staking::Call::increase_validator_count { .. }
				| pallet_staking::Call::scale_validator_count { .. }
				| pallet_staking::Call::force_no_eras { .. }
				| pallet_staking::Call::force_new_era { .. }
				| pallet_staking::Call::set_invulnerables { .. }
				| pallet_staking::Call::force_unstake { .. }
				| pallet_staking::Call::force_new_era_always { .. }
				| pallet_staking::Call::payout_stakers { .. }
				| pallet_staking::Call::rebond { .. }
				| pallet_staking::Call::reap_stash { .. }
				| pallet_staking::Call::set_staking_configs { .. }
				| pallet_staking::Call::chill_other { .. }
				| pallet_staking::Call::force_apply_min_commission { .. },
			)
			| RuntimeCall::Session(pallet_session::Call::purge_keys { .. })
			| RuntimeCall::Grandpa(..)
			| RuntimeCall::ImOnline(..)
			| RuntimeCall::Treasury(..)
			| RuntimeCall::ConvictionVoting(..)
			| RuntimeCall::Referenda(
				pallet_referenda::Call::place_decision_deposit { .. }
				| pallet_referenda::Call::refund_decision_deposit { .. }
				| pallet_referenda::Call::cancel { .. }
				| pallet_referenda::Call::kill { .. }
				| pallet_referenda::Call::nudge_referendum { .. }
				| pallet_referenda::Call::one_fewer_deciding { .. },
			)
			| RuntimeCall::Claims(
				super::claims::Call::claim { .. }
				| super::claims::Call::mint_claim { .. }
				| super::claims::Call::move_claim { .. },
			)
			| RuntimeCall::Utility(pallet_utility::Call::as_derivative { .. })
			| RuntimeCall::Identity(
				pallet_identity::Call::add_registrar { .. }
				| pallet_identity::Call::set_identity { .. }
				| pallet_identity::Call::clear_identity { .. }
				| pallet_identity::Call::request_judgement { .. }
				| pallet_identity::Call::cancel_request { .. }
				| pallet_identity::Call::set_fee { .. }
				| pallet_identity::Call::set_account_id { .. }
				| pallet_identity::Call::set_fields { .. }
				| pallet_identity::Call::provide_judgement { .. }
				| pallet_identity::Call::kill_identity { .. }
				| pallet_identity::Call::add_sub { .. }
				| pallet_identity::Call::rename_sub { .. }
				| pallet_identity::Call::remove_sub { .. }
				| pallet_identity::Call::quit_sub { .. },
			)
			| RuntimeCall::Vesting(..)
			| RuntimeCall::Bounties(
				pallet_bounties::Call::propose_bounty { .. }
				| pallet_bounties::Call::approve_bounty { .. }
				| pallet_bounties::Call::propose_curator { .. }
				| pallet_bounties::Call::unassign_curator { .. }
				| pallet_bounties::Call::accept_curator { .. }
				| pallet_bounties::Call::award_bounty { .. }
				| pallet_bounties::Call::claim_bounty { .. }
				| pallet_bounties::Call::close_bounty { .. },
			)
			| RuntimeCall::ChildBounties(..)
			| RuntimeCall::ElectionProviderMultiPhase(..)
			| RuntimeCall::VoterList(..)
			| RuntimeCall::NominationPools(
				pallet_nomination_pools::Call::join { .. }
				| pallet_nomination_pools::Call::bond_extra { .. }
				| pallet_nomination_pools::Call::claim_payout { .. }
				| pallet_nomination_pools::Call::unbond { .. }
				| pallet_nomination_pools::Call::pool_withdraw_unbonded { .. }
				| pallet_nomination_pools::Call::withdraw_unbonded { .. }
				| pallet_nomination_pools::Call::create { .. }
				| pallet_nomination_pools::Call::create_with_pool_id { .. }
				| pallet_nomination_pools::Call::set_state { .. }
				| pallet_nomination_pools::Call::set_configs { .. }
				| pallet_nomination_pools::Call::update_roles { .. }
				| pallet_nomination_pools::Call::chill { .. },
			)
			| RuntimeCall::Hrmp(..)
			| RuntimeCall::Registrar(
				paras_registrar::Call::deregister { .. }
				| paras_registrar::Call::swap { .. }
				| paras_registrar::Call::remove_lock { .. }
				| paras_registrar::Call::reserve { .. }
				| paras_registrar::Call::add_lock { .. },
			)
			| RuntimeCall::XcmPallet(pallet_xcm::Call::limited_reserve_transfer_assets {
				..
			})
			| RuntimeCall::Whitelist(pallet_whitelist::Call::whitelist_call { .. })
			| RuntimeCall::Proxy(..) => true,
			_ => false,
		}
	}
}
