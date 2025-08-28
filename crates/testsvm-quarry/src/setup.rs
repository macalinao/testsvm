use anyhow::Result;
use testsvm::TestSVM;

use crate::quarry_mine;

/// Setup the quarry programs in the environment
pub fn setup_quarry_programs(env: &mut TestSVM) -> Result<()> {
    env.add_program_fixture("quarry_mine", quarry_mine::ID)?;
    env.add_program_fixture("quarry_merge_mine", crate::quarry_merge_mine::ID)?;
    env.add_program_fixture("quarry_mint_wrapper", crate::quarry_mint_wrapper::ID)?;
    Ok(())
}
