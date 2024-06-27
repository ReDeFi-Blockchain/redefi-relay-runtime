use super::*;

bitflags::bitflags! {
	/// Permissions of an account.
	#[derive(Encode, Decode, MaxEncodedLen, Default, TypeInfo)]
	pub struct AccountPermissions: u64 {
		const MINT = 1;
	}
}
