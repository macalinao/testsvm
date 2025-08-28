//! # TestSVM Quarry
//!
//! Testing utilities for the Quarry protocol on Solana using the TestSVM framework.
//!
//! This crate provides comprehensive testing utilities for interacting with the Quarry mining protocol,
//! including rewarders, miners, merge mining, and mint wrapper functionality. It simplifies the process
//! of testing Quarry-based applications in a controlled environment.
//!
//! ## Features
//!
//! - **Quarry Program Setup**: Easy initialization of all Quarry programs
//! - **Rewarder Management**: Create and manage reward distribution systems
//! - **Mining Operations**: Test single and merge mining functionality
//! - **Mint Wrapper**: Testing utilities for wrapped token minting
//! - **Type-safe Interfaces**: Strongly typed wrappers around Quarry operations
//!
//! ## Prerequisites
//!
//! Before using this crate, download the Quarry program binaries:
//!
//! ```bash
//! # Set your project root
//! export ROOT_DIR=/path/to/your/project
//!
//! # Download Quarry programs
//! solana program dump QMMD16kjauP5knBwxNUJRZ1Z5o3deBuFrqVjBVmmqto \
//!   $ROOT_DIR/fixtures/programs/quarry_merge_mine.so
//!
//! solana program dump QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB \
//!   $ROOT_DIR/fixtures/programs/quarry_mine.so
//!
//! solana program dump QMWoBmAyJLAsA1Lh9ugMTw2gciTihncciphzdNzdZYV \
//!   $ROOT_DIR/fixtures/programs/quarry_mint_wrapper.so
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use testsvm_quarry::{setup_quarry_programs, TestRewarder};
//!
//! // Initialize test environment
//! let mut env = TestSVM::new();
//!
//! // Setup Quarry programs
//! setup_quarry_programs(&mut env)?;
//!
//! // Create a rewarder
//! let authority = env.create_wallet("authority")?;
//! let rewarder = TestRewarder::new(&mut env, &authority)?;
//!
//! // Create a quarry for a token
//! let token_mint = env.create_mint("reward_token", 6)?;
//! let quarry = rewarder.create_quarry(&mut env, &token_mint)?;
//! ```
//!
//! ## Working with Rewarders
//!
//! ```rust,ignore
//! use testsvm_quarry::TestRewarder;
//!
//! // Create a rewarder with custom parameters
//! let rewarder = TestRewarder::new_with_params(
//!     &mut env,
//!     &authority,
//!     1_000_000, // Annual rewards rate
//! )?;
//!
//! // Set annual rewards rate
//! rewarder.set_annual_rewards(&mut env, 2_000_000)?;
//!
//! // Pause/unpause rewards
//! rewarder.pause(&mut env)?;
//! rewarder.unpause(&mut env)?;
//! ```
//!
//! ## Mining Operations
//!
//! ```rust,ignore
//! use testsvm_quarry::{TestQuarry, TestMiner};
//!
//! // Create a miner
//! let miner_authority = env.create_wallet("miner")?;
//! let miner = TestMiner::new(&mut env, &quarry, &miner_authority)?;
//!
//! // Stake tokens
//! let stake_amount = 1_000_000;
//! miner.stake(&mut env, stake_amount)?;
//!
//! // Claim rewards
//! let rewards = miner.claim(&mut env)?;
//! println!("Claimed {} rewards", rewards);
//!
//! // Withdraw staked tokens
//! miner.withdraw(&mut env, stake_amount)?;
//! ```
//!
//! ## Merge Mining
//!
//! ```rust,ignore
//! use testsvm_quarry::{TestMergeMiner, TestMergePool};
//!
//! // Create merge pool
//! let pool = TestMergePool::new(&mut env, &primary_mint)?;
//!
//! // Add replica mint
//! pool.add_replica(&mut env, &replica_mint)?;
//!
//! // Create merge miner
//! let merge_miner = TestMergeMiner::new(
//!     &mut env,
//!     &pool,
//!     &miner_authority,
//! )?;
//!
//! // Stake in primary pool
//! merge_miner.stake_primary(&mut env, 1_000_000)?;
//!
//! // Claim from both primary and replica
//! let (primary_rewards, replica_rewards) = merge_miner.claim_all(&mut env)?;
//! ```
//!
//! ## Mint Wrapper
//!
//! ```rust,ignore
//! use testsvm_quarry::TestMintWrapper;
//!
//! // Create mint wrapper
//! let wrapper = TestMintWrapper::new(
//!     &mut env,
//!     &base_token_mint,
//!     &wrapper_authority,
//! )?;
//!
//! // Wrap tokens
//! let wrapped_amount = wrapper.wrap(&mut env, 1_000_000)?;
//!
//! // Unwrap tokens
//! wrapper.unwrap(&mut env, wrapped_amount)?;
//! ```
//!
//! ## Complete Example
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use testsvm_quarry::*;
//!
//! fn test_quarry_rewards() -> anyhow::Result<()> {
//!     let mut env = TestSVM::new();
//!     
//!     // Setup programs
//!     setup_quarry_programs(&mut env)?;
//!     
//!     // Create accounts
//!     let authority = env.create_wallet("authority")?;
//!     let miner_wallet = env.create_wallet("miner")?;
//!     
//!     // Create tokens
//!     let stake_token = env.create_mint("stake_token", 6)?;
//!     let reward_token = env.create_mint("reward_token", 6)?;
//!     
//!     // Setup rewarder
//!     let rewarder = TestRewarder::new(&mut env, &authority)?;
//!     rewarder.set_annual_rewards(&mut env, 1_000_000)?;
//!     
//!     // Create quarry
//!     let quarry = rewarder.create_quarry(&mut env, &stake_token)?;
//!     
//!     // Create miner and stake
//!     let miner = TestMiner::new(&mut env, &quarry, &miner_wallet)?;
//!     miner.stake(&mut env, 100_000)?;
//!     
//!     // Advance time and claim rewards
//!     env.advance_clock(86400)?; // 1 day
//!     let rewards = miner.claim(&mut env)?;
//!     
//!     assert!(rewards > 0);
//!     Ok(())
//! }
//! ```

use anchor_lang::prelude::*;

pub mod setup;
pub mod test_merge_miner;
pub mod test_merge_pool;
pub mod test_quarry;
pub mod test_rewarder;

pub use setup::*;
pub use test_merge_miner::*;
pub use test_merge_pool::*;
pub use test_quarry::*;
pub use test_rewarder::*;

// Declare quarry programs using their IDLs
declare_program!(quarry_merge_mine);
declare_program!(quarry_mine);
declare_program!(quarry_mint_wrapper);

#[cfg(test)]
mod tests;
