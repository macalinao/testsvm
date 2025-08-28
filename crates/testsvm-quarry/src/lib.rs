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

use anchor_lang::prelude::*;

pub mod prelude;
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
