use spl_tlv_account_resolution::account::ExtraAccountMeta;
use spl_token_2022::solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};
use spl_transfer_hook_interface::instruction::TransferHookInstruction;

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
    program_id: &Pubkey, accounts: &[AccountInfo], extra_account_metas: Vec<ExtraAccountMeta>,
) -> ProgramResult {
    todo!()
}
