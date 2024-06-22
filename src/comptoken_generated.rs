use solana_program::{pubkey, pubkey::Pubkey};

// vvv This line is automatically updated by full_deploy_test.py.
pub static COMPTOKEN_ADDRESS: Pubkey = pubkey!("8ozabKqSf8XjDGTVPsKouwTv6yVc7HuYKTk9bv75pCfj");
// ^^^ DO NOT TOUCH. ^^^

// A given seed and program id have a 50% chance of creating a valid PDA.
// Before building/deploying, we find the canonical seed by running 
//      `solana find-program-derived-address <program_id>`
// This is an efficiency optimization. We are using a static seed to create the PDA with no bump.
// We ensure when deploying that the program id is one that only needs the seed above and no bump.
// This is because 
//      (1) create_program_address is not safe if using a user provided bump.
//      (2) find_program_address is expensive and we want to avoid iterations.
// vvv This line is automatically updated by full_deploy_test.py.
pub static COMPTO_STATIC_ADDRESS_SEED: u8 = 255;
// ^^^ DO NOT TOUCH. ^^^
