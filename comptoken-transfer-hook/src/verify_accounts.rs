use spl_token_2022::solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Debug, Clone)]
pub struct VerifiedAccountInfo<'a>(pub AccountInfo<'a>);

impl<'a> VerifiedAccountInfo<'a> {
    pub fn new(account: AccountInfo<'a>) -> Self {
        Self(account)
    }
}

impl<'a> std::ops::Deref for VerifiedAccountInfo<'a> {
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

fn verify_account_signer_or_writable<'a>(
    account: &AccountInfo<'a>, needs_signer: bool, needs_writable: bool,
) -> VerifiedAccountInfo<'a> {
    // only panic if signing/writing is needed and the account does not meet the requirements
    assert!(!needs_signer || account.is_signer);
    assert!(!needs_writable || account.is_writable);
    VerifiedAccountInfo::new(account.clone())
}

fn verify_pda<'a>(
    account: &AccountInfo<'a>, program_id: &Pubkey, seeds: &[&[u8]], needs_signer: bool, needs_writable: bool,
) -> (VerifiedAccountInfo<'a>, u8) {
    let (result, bump) = Pubkey::find_program_address(seeds, program_id);
    assert_eq!(*account.key, result);
    (verify_account_signer_or_writable(account, needs_signer, needs_writable), bump)
}

pub fn verify_validation_account<'a>(
    account: &AccountInfo<'a>, mint: &VerifiedAccountInfo, program_id: &Pubkey, needs_writable: bool,
) -> (VerifiedAccountInfo<'a>, u8) {
    verify_pda(account, program_id, &crate::get_validation_account_seeds(mint), false, needs_writable)
}

pub fn verify_mint_account<'a>(account: &AccountInfo<'a>) -> VerifiedAccountInfo<'a> {
    // TODO verify mint
    verify_account_signer_or_writable(account, false, false)
}
