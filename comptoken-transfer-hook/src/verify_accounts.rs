use spl_token_2022::{
    extension::StateWithExtensions,
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
    state::Mint,
};

pub use comptoken_utils::verify_accounts::VerifiedAccountInfo;

pub fn verify_account_meta_storage_account<'a>(
    account: &AccountInfo<'a>, mint: &VerifiedAccountInfo, program_id: &Pubkey, needs_writable: bool,
) -> (VerifiedAccountInfo<'a>, u8) {
    VerifiedAccountInfo::verify_pda(
        account,
        program_id,
        &crate::get_account_meta_storage_account_seeds(mint),
        false,
        needs_writable,
    )
}

pub fn verify_mint_account<'a>(account: &AccountInfo<'a>) -> VerifiedAccountInfo<'a> {
    VerifiedAccountInfo::verify_account_signer_or_writable(account, false, false)
}

pub fn verify_mint_authority<'a>(
    account: &AccountInfo<'a>, mint: &VerifiedAccountInfo, needs_signer: bool, needs_writable: bool,
) -> VerifiedAccountInfo<'a> {
    let data = mint.try_borrow_data().unwrap();
    let mint = StateWithExtensions::<Mint>::unpack(&data).unwrap();
    assert_eq!(*account.key, mint.base.mint_authority.expect("has a mint authority"));
    VerifiedAccountInfo::verify_account_signer_or_writable(account, needs_signer, needs_writable)
}