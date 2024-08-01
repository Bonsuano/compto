use std::ops::Deref;

use solana_program::sysvar::SysvarId;
use spl_token_2022::solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Debug, Clone)]
pub struct VerifiedAccountInfo<'a>(pub AccountInfo<'a>);

impl<'a> VerifiedAccountInfo<'a> {
    pub fn new(account: AccountInfo<'a>) -> Self {
        Self(account)
    }

    fn verify_account_signer_or_writable(account: &AccountInfo<'a>, needs_signer: bool, needs_writable: bool) -> Self {
        // only panic if signing/writing is needed and the account does not meet the requirements
        assert!(!needs_signer || account.is_signer);
        assert!(!needs_writable || account.is_writable);
        VerifiedAccountInfo::new(account.clone())
    }

    pub fn verify_pda(
        account: &AccountInfo<'a>, program_id: &Pubkey, seeds: &[&[u8]], needs_signer: bool, needs_writable: bool,
    ) -> (Self, u8) {
        let (result, bump) = Pubkey::find_program_address(seeds, program_id);
        assert_eq!(*account.key, result);
        (Self::verify_account_signer_or_writable(account, needs_signer, needs_writable), bump)
    }

    pub fn verify_sysvar<S: SysvarId>(account: &AccountInfo<'a>) -> Self {
        assert!(S::check_id(account.key));
        Self::new(account.clone())
    }

    pub fn verify_payer_account(account: &AccountInfo<'a>) -> Self {
        Self::verify_account_signer_or_writable(account, true, true)
    }

    pub fn verify_specific_account(
        account: &AccountInfo<'a>, address: &Pubkey, needs_signer: bool, needs_writable: bool,
    ) -> Self {
        assert_eq!(account.key, address);
        Self::verify_account_signer_or_writable(account, needs_signer, needs_writable)
    }
}

impl<'a> Deref for VerifiedAccountInfo<'a> {
    type Target = AccountInfo<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<VerifiedAccountInfo<'a>> for AccountInfo<'a> {
    fn from(val: VerifiedAccountInfo<'a>) -> Self {
        val.0
    }
}
