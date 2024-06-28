use {
    crate::{error::StakeError, processor},
    solana_program::{
        account_info::AccountInfo,
        entrypoint::{self, ProgramResult},
        program_error::PrintProgramError,
        pubkey::Pubkey,
    },
};

entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process_instruction(program_id, accounts, instruction_data) {
        error.print::<StakeError>();
        return Err(error);
    }

    Ok(())
}
