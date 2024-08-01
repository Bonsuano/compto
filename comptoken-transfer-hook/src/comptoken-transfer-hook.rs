mod verify_accounts;

use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList};
use spl_token_2022::solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_transfer_hook_interface::instruction::{ExecuteInstruction, TransferHookInstruction};

use comptoken_utils::create_pda;

use verify_accounts::{verify_mint_account, verify_mint_authority, verify_validation_account, VerifiedAccountInfo};

entrypoint!(process_instruction);
pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    match TransferHookInstruction::unpack(instruction_data)? {
        TransferHookInstruction::Execute { amount } => process_execute(program_id, accounts, amount),
        TransferHookInstruction::InitializeExtraAccountMetaList { extra_account_metas } => {
            process_initialize_extra_account_meta_list(program_id, accounts, extra_account_metas)
        }
        TransferHookInstruction::UpdateExtraAccountMetaList { extra_account_metas: _ } => {
            panic!("instruction not implemented");
        }
    }
}

fn process_execute(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!()
}

fn process_initialize_extra_account_meta_list(
    program_id: &Pubkey, accounts: &[AccountInfo], _extra_account_metas: Vec<ExtraAccountMeta>,
) -> ProgramResult {
    //      [writable]: Validation account
    //      []: Mint
    //      [signer]: Mint authority
    //      []: System program
    //      [signer, writable]: payer account (not part of the standard)

    let account_info_iter = &mut accounts.iter();
    let validation_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let mint_authority = next_account_info(account_info_iter)?;
    let _system_program = next_account_info(account_info_iter)?;
    let payer_account = next_account_info(account_info_iter)?;

    let mint_account = verify_mint_account(mint_account);
    let (validation_account, validation_account_bump) =
        verify_validation_account(validation_account, &mint_account, program_id, true);
    let _mint_authority = verify_mint_authority(mint_authority, &mint_account, true, false);
    let payer_account = VerifiedAccountInfo::verify_account_signer_or_writable(payer_account, true, true);

    let account_metas = vec![
        ExtraAccountMeta::new_with_seeds(&[Seed::AccountKey { index: 0 }], false, false)?,
        ExtraAccountMeta::new_with_seeds(&[Seed::AccountKey { index: 2 }], false, false)?,
    ];

    let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;

    let lamports = Rent::get()?.minimum_balance(account_size as usize);

    let mut validation_account_seeds = get_validation_account_seeds(&mint_account);
    validation_account_seeds.push(std::array::from_ref(&validation_account_bump));

    let signer_seeds: &[&[&[u8]]] = &[&validation_account_seeds];

    create_pda(&payer_account, &validation_account, lamports, account_size, program_id, signer_seeds)?;

    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut validation_account.try_borrow_mut_data()?, &account_metas)?;

    Ok(())
}

fn get_validation_account_seeds<'a>(mint: &'a VerifiedAccountInfo) -> Vec<&'a [u8]> {
    vec![b"extra-account-metas", mint.key.as_ref()]
}
