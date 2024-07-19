use spl_token_2022::solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::generated::{
    COMPTOKEN_MINT_ADDRESS, COMPTO_GLOBAL_DATA_ACCOUNT_SEEDS, COMPTO_INTEREST_BANK_ACCOUNT_SEEDS,
    COMPTO_UBI_BANK_ACCOUNT_SEEDS,
};

#[derive(Debug, Clone)]
pub struct VerifiedAccountInfo<'a>(pub AccountInfo<'a>);

impl<'a> VerifiedAccountInfo<'a> {
    pub fn new<'b>(account: &'b AccountInfo) -> &'b Self {
        unsafe { &*(account as *const _ as *const Self) }
    }
}

impl<'a> Into<AccountInfo<'a>> for VerifiedAccountInfo<'a> {
    fn into(self) -> AccountInfo<'a> {
        self.0
    }
}

fn verify_account_signer_or_writable<'a, 'b>(
    account: &'b AccountInfo<'a>, needs_signer: bool, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    assert!(!needs_signer || account.is_signer);
    assert!(!needs_writable || account.is_writable);
    VerifiedAccountInfo::new(account)
}

pub fn verify_payer_account<'a, 'b>(account: &'b AccountInfo<'a>) -> &'b VerifiedAccountInfo<'a> {
    verify_account_signer_or_writable(account, true, true)
}

pub fn verify_comptoken_mint<'a, 'b>(
    account: &'b AccountInfo<'a>, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    assert_eq!(*account.key, COMPTOKEN_MINT_ADDRESS);
    verify_account_signer_or_writable(account, false, needs_writable)
}

pub fn verify_global_data_account<'a, 'b>(
    account: &'b AccountInfo<'a>, program_id: &Pubkey, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    let result = Pubkey::create_program_address(COMPTO_GLOBAL_DATA_ACCOUNT_SEEDS, program_id).unwrap();
    assert_eq!(*account.key, result);
    verify_account_signer_or_writable(account, false, needs_writable)
}

pub fn verify_interest_bank_account<'a, 'b>(
    account: &'b AccountInfo<'a>, program_id: &Pubkey, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    let result = Pubkey::create_program_address(COMPTO_INTEREST_BANK_ACCOUNT_SEEDS, program_id).unwrap();
    assert_eq!(*account.key, result);
    verify_account_signer_or_writable(account, false, needs_writable)
}

pub fn verify_ubi_bank_account<'a, 'b>(
    account: &'b AccountInfo<'a>, program_id: &Pubkey, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    let result = Pubkey::create_program_address(COMPTO_UBI_BANK_ACCOUNT_SEEDS, program_id).unwrap();
    assert_eq!(*account.key, result);
    verify_account_signer_or_writable(account, false, needs_writable)
}

pub fn verify_user_comptoken_wallet_account<'a, 'b>(
    account: &'b AccountInfo<'a>, needs_signer: bool, needs_writable: bool,
) -> &'b VerifiedAccountInfo<'a> {
    // TODO: verify comptoken user wallet accounts
    verify_account_signer_or_writable(account, needs_signer, needs_writable)
}

pub fn verify_user_data_account<'a, 'b>(
    user_data_account: &'b AccountInfo<'a>, user_comptoken_wallet_account: &VerifiedAccountInfo, program_id: &Pubkey,
    needs_writable: bool,
) -> (&'b VerifiedAccountInfo<'a>, u8) {
    // if we ever need a user data account to sign something,
    // then we should return the bumpseed in this function
    let (pda, bump) = Pubkey::find_program_address(&[user_comptoken_wallet_account.0.key.as_ref()], program_id);
    assert_eq!(*user_data_account.key, pda, "Invalid user data account");
    (verify_account_signer_or_writable(user_data_account, false, needs_writable), bump)
}

pub fn verify_slothashes_account<'a, 'b>(account: &'b AccountInfo<'a>) -> &'b VerifiedAccountInfo<'a> {
    assert!(solana_program::sysvar::slot_hashes::check_id(account.key));
    VerifiedAccountInfo::new(account)
}
