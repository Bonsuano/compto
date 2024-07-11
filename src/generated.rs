// full_deploy_test.py generates a comptoken_generated.rs
// The first build must not have the testmode feature enabled so that a ProgramId is created.
// full_deploy_test.py handles this case gracefully by building twice on the first usage.
#[cfg(feature = "testmode")]
mod comptoken_generated;
#[cfg(not(feature = "testmode"))]
mod comptoken_generated {
    use spl_token_2022::solana_program::{pubkey, pubkey::Pubkey};
    pub const COMPTOKEN_MINT_ACCOUNT_ADDRESS: Pubkey = pubkey!("11111111111111111111111111111111");
    pub const COMPTO_GLOBAL_DATA_ACCOUNT_SEED: u8 = 255;
}
pub use comptoken_generated::{COMPTOKEN_MINT_ACCOUNT_ADDRESS, COMPTO_GLOBAL_DATA_ACCOUNT_SEED};

pub const COMPTO_GLOBAL_DATA_ACCOUNT_SEEDS: &[&[u8]] = &[&[COMPTO_GLOBAL_DATA_ACCOUNT_SEED]];
