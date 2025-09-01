//! # TestSVM Core Prelude
//!
//! Common imports for TestSVM users. This module re-exports the most commonly used types
//! and traits from testsvm-core for convenient access.

// Core TestSVM types
pub use crate::{AccountRef, TXError, TXResult, TestSVM};

// Address book types
pub use solana_address_book::{AddressBook, AddressRole, RegisteredAddress};

// Commonly used Anchor types
pub use anchor_lang::{
    AccountDeserialize, AccountSerialize, Discriminator, InstructionData, Key, ToAccountInfos,
    ToAccountMetas,
};

pub use anyhow::Result;

// Commonly used Solana SDK types
pub use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::Transaction,
};

// Re-export anchor_instruction helper from anchor-utils
pub use anchor_utils::anchor_instruction;
