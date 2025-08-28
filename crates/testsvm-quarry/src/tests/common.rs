use anyhow::Result;
use testsvm::prelude::*;

use crate::setup_quarry_programs;

pub fn init_test_environment() -> Result<TestSVM> {
    let mut env = TestSVM::init()?;
    setup_quarry_programs(&mut env)?;
    Ok(env)
}
