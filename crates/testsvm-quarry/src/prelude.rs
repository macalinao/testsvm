//! # TestSVM Quarry Prelude
//!
//! Common imports for testing Quarry protocol applications.
//!
//! This module re-exports the most commonly used types from the testsvm-quarry
//! crate along with TestSVM core functionality, providing everything needed
//! for testing Quarry-based applications with a single import:
//!
//! ```rust
//! use testsvm_quarry::prelude::*;
//! ```
//!
//! ## Included Exports
//!
//! - **Quarry Test Types**: Test helpers for rewarder, quarry, miners, and pools
//! - **TestSVM Core**: All exports from testsvm::prelude
//! - **Quarry Programs**: Generated types from declare_program! macros
//! - **Setup Functions**: Helper functions for program initialization

// Re-export everything from testsvm prelude
pub use testsvm::prelude::*;

// Quarry test helpers
pub use crate::{TestMergeMiner, TestMergePool, TestQuarry, TestRewarder};

// Setup functions
pub use crate::setup_quarry_programs;

// Quarry program types (generated from declare_program!)
pub use crate::{quarry_merge_mine, quarry_mine, quarry_mint_wrapper};
