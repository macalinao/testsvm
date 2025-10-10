//! # TestSVM Core
//!
//! Core implementation of the TestSVM testing framework for Solana programs.
//!
//! This crate provides the fundamental `TestSVM` struct that wraps LiteSVM
//! with additional functionality for transaction management, account creation,
//! and enhanced debugging capabilities.

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::*;
use litesvm::LiteSVM;
use solana_sdk::{
    clock::Clock,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub use solana_address_book::AddressBook;

mod tx_result;
pub use tx_result::{TXError, TXResult};

mod account_ref;
pub use account_ref::AccountRef;

mod litesvm_helpers;
use litesvm_helpers::new_funded_account;

pub mod prelude;

/// Test SVM wrapper for LiteSVM with payer management and Anchor helpers
pub struct TestSVM {
    /// Underlying LiteSVM instance
    pub svm: LiteSVM,
    /// Default fee payer for transactions.
    pub default_fee_payer: Keypair,
    /// Address book for labeling addresses
    pub address_book: AddressBook,
}

impl TestSVM {
    /// Create a new test SVM with a payer and address book
    pub fn init() -> Result<Self> {
        let mut svm = LiteSVM::new();
        let default_fee_payer = new_funded_account(&mut svm, 1000 * 1_000_000_000)?;
        let mut address_book = AddressBook::new();
        address_book.add_default_accounts()?;

        // Add the default fee payer to the address book
        address_book.add_wallet(default_fee_payer.pubkey(), "default_fee_payer".to_string())?;

        Ok(Self {
            svm,
            default_fee_payer,
            address_book,
        })
    }

    /// Execute a transaction with the test SVM's payer
    pub fn execute_transaction(&mut self, transaction: Transaction) -> TXResult {
        match self.svm.send_transaction(transaction.clone()) {
            Result::Ok(tx_result) => Result::Ok(tx_result),
            Err(e) => Err(Box::new(TXError {
                transaction,
                metadata: e.clone(),
                address_book: self.address_book.clone(),
            })),
        }
    }

    /// Execute instructions with the test SVM's payer
    pub fn execute_ixs(
        &mut self,
        instructions: &[solana_sdk::instruction::Instruction],
    ) -> TXResult {
        self.execute_ixs_with_signers(instructions, &[])
    }

    /// Execute instructions with additional signers
    pub fn execute_ixs_with_signers(
        &mut self,
        instructions: &[solana_sdk::instruction::Instruction],
        signers: &[&Keypair],
    ) -> TXResult {
        let mut all_signers = vec![&self.default_fee_payer];
        all_signers.extend_from_slice(signers);

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.default_fee_payer.pubkey()),
            &all_signers,
            self.svm.latest_blockhash(),
        );

        self.execute_transaction(transaction)
    }

    /// Create a new funded wallet and add to address book
    pub fn new_wallet(&mut self, name: &str) -> Result<Keypair> {
        let keypair = new_funded_account(&mut self.svm, 10 * 1_000_000_000)?; // 10 SOL
        let label = format!("wallet:{name}");
        self.address_book.add_wallet(keypair.pubkey(), label)?;
        Ok(keypair)
    }

    /// Get the default fee payer's public key
    pub fn default_fee_payer(&self) -> Pubkey {
        self.default_fee_payer.pubkey()
    }

    /// Add a program to the address book
    pub fn add_program_from_path(
        &mut self,
        label: &str,
        pubkey: Pubkey,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        self.svm.add_program_from_file(pubkey, path)?;
        self.address_book.add_program(pubkey, label)
    }

    /// Add a program fixture from the fixtures directory.
    ///
    /// This method loads a program binary from the fixtures directory. The fixture file
    /// should be located at `fixtures/programs/{fixture_name}.so` relative to your project root.
    pub fn add_program_fixture(&mut self, fixture_name: &str, pubkey: Pubkey) -> Result<()> {
        let path = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map_err(|e| anyhow!("Failed to get environment variable `CARGO_MANIFEST_DIR`: {e}"))?
            .ancestors()
            .find_map(|ancestor| {
                let fixtures_dir = ancestor.join("fixtures");
                fixtures_dir.exists().then_some(fixtures_dir)
            })
            .ok_or_else(|| anyhow!("`fixtures` directory not found"))
            .map(|fixtures_dir| {
                fixtures_dir
                    .join("programs")
                    .join(fixture_name)
                    .with_extension("so")
            })?;

        self.add_program_from_path(fixture_name, pubkey, &path)?;
        Ok(())
    }

    /// Finds a program derived address and return an [AccountRef] with proper type information.
    pub fn get_pda<T: anchor_lang::AccountDeserialize>(
        &mut self,
        label: &str,
        seeds: &[&[u8]],
        program_id: Pubkey,
    ) -> Result<AccountRef<T>> {
        let (pda, _) = self.get_pda_with_bump(label, seeds, program_id)?;
        Ok(pda)
    }

    /// Finds a program derived address and return an [AccountRef] with proper type information and bump seed.
    pub fn get_pda_with_bump<T: anchor_lang::AccountDeserialize>(
        &mut self,
        label: &str,
        seeds: &[&[u8]],
        program_id: Pubkey,
    ) -> Result<(AccountRef<T>, u8)> {
        let (pubkey, bump) = self
            .address_book
            .find_pda_with_bump(label, seeds, program_id)?;
        Ok((AccountRef::new(pubkey), bump))
    }

    /// Advance the time by the specified number of seconds
    /// Assumes 450ms per slot, in practice this is not always the case.
    pub fn advance_time(&mut self, seconds: u64) {
        let mut clock = self.svm.get_sysvar::<Clock>();
        clock.unix_timestamp += seconds as i64;
        // assume 450ms per slot.
        let num_slots = seconds / 450;
        clock.slot += num_slots;
        self.svm.set_sysvar(&clock);
    }

    /// Advance slots using LiteSVM's warp_to_slot feature
    /// This is useful for simulating time passing in tests
    pub fn advance_slots(&mut self, num_slots: u32) {
        let current_slot = self.svm.get_sysvar::<solana_sdk::clock::Clock>().slot;
        let target_slot = current_slot + num_slots as u64;

        self.svm.warp_to_slot(target_slot);
    }
}
