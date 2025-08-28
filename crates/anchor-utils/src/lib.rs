use anchor_lang::prelude::*;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::instruction::Instruction;

/// Creates a new Anchor instruction from the generated `declare_program!` client structs
pub fn anchor_instruction(
    program_id: Pubkey,
    accounts: impl ToAccountMetas,
    data: impl InstructionData,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}
