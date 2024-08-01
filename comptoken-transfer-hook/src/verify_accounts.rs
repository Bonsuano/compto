use spl_token_2022::solana_program::{account_info::AccountInfo, pubkey::Pubkey};

pub use comptoken_utils::verify_accounts::VerifiedAccountInfo;

pub fn verify_validation_account<'a>(
    account: &AccountInfo<'a>, mint: &VerifiedAccountInfo, program_id: &Pubkey, needs_writable: bool,
) -> (VerifiedAccountInfo<'a>, u8) {
    VerifiedAccountInfo::verify_pda(
        account,
        program_id,
        &crate::get_validation_account_seeds(mint),
        false,
        needs_writable,
    )
}

pub fn verify_mint_account<'a>(account: &AccountInfo<'a>) -> VerifiedAccountInfo<'a> {
    // TODO verify mint
    VerifiedAccountInfo::verify_account_signer_or_writable(account, false, false)
}
