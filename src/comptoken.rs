mod comptoken_proof;

extern crate bs58;

use comptoken_proof::ComptokenProof;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    hash::Hash,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction::create_account,
    sysvar::slot_history::ProgramError,
};
use spl_token::instruction::mint_to;
// declare and export the program's entrypoint
entrypoint!(process_instruction);

// MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN test_client.js
const STATIC_ACCOUNT_SPACE: u64 = 4096;

// full_deploy_test.py generates a comptoken_generated.rs
// The first build must not have the testmode feature enabled so that a ProgramId is created.
// full_deploy_test.py handles this case gracefully by building twice on the first usage.
#[cfg(feature = "testmode")]
mod comptoken_generated;
#[cfg(not(feature = "testmode"))]
mod comptoken_generated {
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const COMPTOKEN_ADDRESS: Pubkey = pubkey!("11111111111111111111111111111111");
    pub const COMPTO_STATIC_ADDRESS_SEED: u8 = 255;
}
use comptoken_generated::{COMPTOKEN_ADDRESS, COMPTO_STATIC_ADDRESS_SEED};

const COMPTO_STATIC_PDA_SEEDS: &[&[u8]] = &[&[COMPTO_STATIC_ADDRESS_SEED]];

// #[derive(Debug, Default, BorshDeserialize, BorshSerialize)]
// pub struct DataAccount {
//     pub hash: [u8; 32], // Assuming you want to store a 32-byte hash
// }

// program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("instruction_data: {:?}", instruction_data);
    match instruction_data[0] {
        0 => {
            msg!("Test Mint");
            test_mint(program_id, accounts, &instruction_data[1..])
        }
        1 => {
            msg!("Mint New Comptokens");
            mint_comptokens(program_id, accounts, &instruction_data[1..])
        }
        2 => {
            msg!("Initialize Static Data Account");
            initialize_static_data_account(program_id, accounts, &instruction_data[1..])
        }
        _ => {
            msg!("Invalid Instruction");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

pub fn initialize_static_data_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //  accounts order:
    //      owner id
    //      mint authority? pda

    msg!("instruction_data: {:?}", instruction_data);

    let account_info_iter = &mut accounts.iter();
    let owner_account = next_account_info(account_info_iter)?;
    let mint_authority_account = next_account_info(account_info_iter)?;

    // verify_owner_account(owner_account)?;
    verify_mint_authority_account(mint_authority_account, program_id)?;

    let first_8_bytes: [u8; 8] = instruction_data[0..8].try_into().unwrap();
    let lamports = u64::from_be_bytes(first_8_bytes);
    msg!("Lamports: {:?}", lamports);

    let create_acct_instr = create_account(
        owner_account.key,
        &mint_authority_account.key,
        lamports,
        STATIC_ACCOUNT_SPACE,
        program_id,
    );
    // let createacct = SystemInstruction::CreateAccount { lamports: (1000), space: (256), owner: *program_id };
    let result = invoke_signed(&create_acct_instr, accounts, &[COMPTO_STATIC_PDA_SEEDS])?;
    // let data = accounts[0].try_borrow_mut_data()?;
    // data[0] = 1;
    Ok(())
}

fn mint(
    mint_pda: &Pubkey,
    destination: &Pubkey,
    amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let instruction = mint_to(
        &spl_token::id(),
        &COMPTOKEN_ADDRESS,
        &destination,
        &mint_pda,
        &[&mint_pda],
        amount,
    )?;
    invoke_signed(&instruction, accounts, &[COMPTO_STATIC_PDA_SEEDS])
}

fn verify_destination_account(account: &AccountInfo) -> ProgramResult {
    // TODO: verify account
    Ok(())
}

fn verify_mint_authority_account(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    // TODO: is this correct
    if *account.key != Pubkey::create_program_address(COMPTO_STATIC_PDA_SEEDS, program_id)? {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}

fn verify_token_account(account: &AccountInfo) -> ProgramResult {
    if *account.key != spl_token::id() {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}

fn verify_comptoken_account(account: &AccountInfo) -> ProgramResult {
    if *account.key != COMPTOKEN_ADDRESS {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}

pub fn test_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //  accounts order:
    //      destination comptoken account
    //      mint authority account
    //      spl_token account
    //      comptoken program account

    msg!("instruction_data: {:?}", instruction_data);
    for account_info in accounts.iter() {
        msg!("Public Key: {:?}", account_info.key);
    }

    let account_info_iter = &mut accounts.iter();
    let destination_account = next_account_info(account_info_iter)?;
    let mint_authority_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let comptoken_account = next_account_info(account_info_iter)?;

    verify_destination_account(destination_account)?;
    verify_mint_authority_account(mint_authority_account, program_id)?;
    verify_token_account(token_account)?;
    verify_comptoken_account(comptoken_account)?;

    let amount = 2;

    //let destination_pubkey = accounts[0].key;
    // Create the mint_to instruction
    //let mint_pda = Pubkey::create_program_address(COMPTO_STATIC_PDA_SEEDS, &program_id)?;
    //msg!("Mint PDA: {:?}", mint_pda);
    // msg!("bump: {:?}", bump);
    mint(
        mint_authority_account.key,
        destination_account.key,
        amount,
        accounts,
    )
    //let mint_to_instruction = mint_to(
    //    &spl_token::id(),
    //    &COMPTOKEN_ADDRESS,
    //    &destination_pubkey,
    //    &mint_pda,
    //    &[&mint_pda],
    //    amount,
    //)?;
    //// accounts.push(AccountInfo::new(&mint_pda, true, true));
    //// Invoke the token program
    //let result = invoke_signed(&mint_to_instruction, accounts, &[COMPTO_STATIC_PDA_SEEDS])?;
    //// msg!("Result: {:?}", result);
    //// gracefully exit the program
    //Ok(())
}

fn verify_data_mint_comptokens(destination: &Pubkey, data: &[u8]) -> Result<Hash, ProgramError> {
    if data.len() != comptoken_proof::VERIFY_DATA_SIZE {
        msg!("invalid instruction data");
        Err(ProgramError::InvalidInstructionData)
    } else {
        let block = ComptokenProof::from_bytes(destination, data.try_into().expect("correct size"));
        msg!("block: {:?}", block);
        if !comptoken_proof::verify_proof(&block) {
            msg!("invalid proof");
            Err(ProgramError::InvalidArgument)
        } else {
            Ok(block.hash)
        }
    }
}

pub fn mint_comptokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //  accounts order:
    //      destination pda
    //      mint authority pda
    //      spl_token id
    //      comptoken id

    //msg!("instruction_data: {:?}", instruction_data);
    //for account_info in accounts.iter() {
    //    msg!("Public Key: {:?}", account_info.key);
    //}

    let account_info_iter = &mut accounts.iter();
    let destination_account = next_account_info(account_info_iter)?;
    let mint_authority_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let comptoken_account = next_account_info(account_info_iter)?;

    verify_destination_account(destination_account)?;
    verify_mint_authority_account(mint_authority_account, program_id)?;
    verify_token_account(token_account)?;
    verify_comptoken_account(comptoken_account)?;

    let hash = verify_data_mint_comptokens(destination_account.key, instruction_data)?;
    let amount = 2;

    msg!("Hash: {:?}", hash);
    //test_mint(program_id, accounts, instruction_data)?;
    mint(
        mint_authority_account.key,
        destination_account.key,
        amount,
        accounts,
    )?;
    // now save the hash to the account

    todo!("implement minting and storing of hashing");
    Ok(())
}
