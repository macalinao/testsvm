//! # TestSVM Prelude
//!
//! Common imports for TestSVM users.
//!
//! This module re-exports the most commonly used types and traits from the TestSVM
//! framework, allowing users to import everything they need with a single use statement:
//!
//! ```rust
//! use testsvm::prelude::*;
//! ```

// Core TestSVM types
pub use crate::{
    AccountRef, TXError, TXErrorAssertions, TXResult, TXResultAssertions, TXSuccessAssertions,
    TestSVM,
};

// SPL helpers
pub use crate::TestSVMSPLHelpers;

// Address book exports
pub use crate::{AddressBook, AddressRole, RegisteredAddress};

// PDA utilities
pub use crate::DerivedPda;

// Helper functions
pub use crate::anchor_instruction;

// Re-export commonly used Solana SDK types
pub use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::Transaction,
};

// Re-export Anchor types that are commonly used
pub use anchor_lang::{InstructionData, Key, ToAccountMetas, prelude::*};
pub use anchor_spl;
