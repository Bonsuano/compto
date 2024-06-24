mod blockchain;

extern crate bs58;

use blockchain::Block;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    hash::Hash,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction::create_account,
    // sysvar::{slot_hashes::SlotHashes, Sysvar},
    sysvar,
    sysvar::slot_history::ProgramError,
};
use spl_token::instruction::mint_to;
// declare and export the program's entrypoint
entrypoint!(process_instruction);

mod comptoken_generated;
use comptoken_generated::{COMPTOKEN_ADDRESS, COMPTO_STATIC_ADDRESS_SEED};

// #[derive(Debug, Default, BorshDeserialize, BorshSerialize)]
// pub struct DataAccount {
//     pub hash: [u8; 32], // Assuming you want to store a 32-byte hash
// }

#[repr(u8)]
enum ComptokenInstructions {
    TestMint = 0,
    MintComptoken = 1,
    InitializeStaticDataAccount = 2,
}

impl TryFrom<u8> for ComptokenInstructions {
    type Error = ProgramError;
    fn try_from(num: u8) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(Self::TestMint),
            1 => Ok(Self::MintComptoken),
            2 => Ok(Self::InitializeStaticDataAccount),
            _ => {
                msg!("Invalid Instruction");
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}

// program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("instruction_data: {:?}", instruction_data);
    match instruction_data[0].try_into()? {
        ComptokenInstructions::TestMint => {
            msg!("Test Mint");
            test_mint(program_id, accounts, &instruction_data[1..])
        }
        ComptokenInstructions::MintComptoken => {
            msg!("Mint New Comptokens");
            mint_comptokens(program_id, accounts, &instruction_data[1..])
        }
        ComptokenInstructions::InitializeStaticDataAccount => {
            msg!("Initialize Static Data Account");
            initialize_static_data_account(program_id, accounts, &instruction_data[1..])
        }
    }
}

pub fn initialize_static_data_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mint_pda = Pubkey::create_program_address(&[&[COMPTO_STATIC_ADDRESS_SEED]], &program_id)?;
    assert_eq!(accounts[0].key, &mint_pda, "Invalid Mint PDA account.");

    msg!("instruction_data: {:?}", instruction_data);

    // let initialize_account_instruction =
    let first_8_bytes: [u8; 8] = instruction_data[0..8].try_into().unwrap();
    let lamports = u64::from_le_bytes(first_8_bytes);
    msg!("Lamports: {:?}", lamports);
    let create_acct_instr = create_account(
        accounts[1].key,
        &mint_pda,
        lamports,
        // MAGIC NUMBER: CHANGE NEEDS TO BE REFLECTED IN test_client.js
        4096,
        program_id,
    );
    // let createacct = SystemInstruction::CreateAccount { lamports: (1000), space: (256), owner: *program_id };
    let result = invoke_signed(
        &create_acct_instr,
        accounts,
        &[&[&[COMPTO_STATIC_ADDRESS_SEED]]],
    )?;
    // let data = accounts[0].try_borrow_mut_data()?;
    // data[0] = 1;
    Ok(())
}

// struct ComptokenMintProof {
//     sh
// }

pub fn test_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("instruction_data: {:?}", instruction_data);
    let amount = 2;
    for account_info in accounts.iter() {
        msg!("Public Key: {:?}", account_info.key);
    }
    let destination_pubkey = accounts[0].key;
    // Create the mint_to instruction
    let mint_pda = Pubkey::create_program_address(&[&[COMPTO_STATIC_ADDRESS_SEED]], &program_id)?;
    msg!("Mint PDA: {:?}", mint_pda);
    // msg!("bump: {:?}", bump);
    let mint_to_instruction = mint_to(
        &spl_token::id(),
        &COMPTOKEN_ADDRESS,
        &destination_pubkey,
        &mint_pda,
        &[&mint_pda],
        amount,
    )?;
    // accounts.push(AccountInfo::new(&mint_pda, true, true));
    // Invoke the token program
    let result = invoke_signed(
        &mint_to_instruction,
        accounts,
        &[&[&[COMPTO_STATIC_ADDRESS_SEED]]],
    )?;
    // msg!("Result: {:?}", result);
    // gracefully exit the program
    Ok(())
}

pub fn mint_comptokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // this nonce is what the miner increments to find a valid proof
    if instruction_data.len() != 32 + 32 + 4 + 32 {
        msg!("invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    let account_info_iter = &mut accounts.iter();
    let first_acc_info = next_account_info(account_info_iter)?; // 0
    if !first_acc_info.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !blockchain::verify_proof(Block::from_bytes(
        first_acc_info.key.clone(),
        instruction_data.try_into().expect("correct size"),
    )) {
        msg!("invalid proof");
        return Err(ProgramError::InvalidArgument);
    }
    // let nonce = instruction_data[..32].try_into().unwrap();
    // verify_proof(accounts[1].key, nonce);

    // get_pseudo_random();

    assert_eq!(
        accounts[0].key,
        &sysvar::slot_hashes::id(),
        "Invalid SlotHashes account."
    );
    let data = accounts[0].try_borrow_data()?;
    let hash = Hash::new(&data[16..48]);
    msg!("Hash: {:?}", hash);
    // now save the hash to the account

    // let mint_pda = Pubkey::create_program_address(&[&[COMPTO_STATIC_ADDRESS_SEED]], &program_id)?;
    // let mut pda_data = mint_pda.try_borrow_mut_data()?;
    // pda_data[0].copy_from_slice(instruction_data[0]);
    // msg!("data: {:?}", encode(&data[..64]));
    Ok(())
}
