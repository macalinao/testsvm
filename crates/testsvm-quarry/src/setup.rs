//! # Quarry Program Setup
//!
//! Utilities for initializing Quarry protocol programs in test environments.
//!
//! This module provides functions to easily set up all required Quarry programs
//! (mine, merge_mine, and mint_wrapper) in a TestSVM environment. These programs
//! must be downloaded as `.so` files before they can be loaded into the test environment.
//!
//! ## Required Programs
//!
//! - **quarry_mine**: Core mining and rewards distribution
//! - **quarry_merge_mine**: Merge mining functionality for multiple quarries
//! - **quarry_mint_wrapper**: Wrapped token minting capabilities

use anyhow::Result;
use testsvm::TestSVM;

use crate::quarry_mine;

/// Setup the quarry programs in the environment.
///
/// Note: you will need to download the Quarry programs to your `fixtures/programs/` directory.
///
/// You can use the following commands:
/// ```bash
/// solana program dump QMMD16kjauP5knBwxNUJRZ1Z5o3deBuFrqVjBVmmqto $ROOT_DIR/fixtures/programs/quarry_merge_mine.so
/// solana program dump QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB $ROOT_DIR/fixtures/programs/quarry_mine.so
/// solana program dump QMWoBmAyJLAsA1Lh9ugMTw2gciTihncciphzdNzdZYV $ROOT_DIR/fixtures/programs/quarry_mint_wrapper.so
/// ```
pub fn setup_quarry_programs(env: &mut TestSVM) -> Result<()> {
    env.add_program_fixture("quarry_mine", quarry_mine::ID)?;
    env.add_program_fixture("quarry_merge_mine", crate::quarry_merge_mine::ID)?;
    env.add_program_fixture("quarry_mint_wrapper", crate::quarry_mint_wrapper::ID)?;
    Ok(())
}
