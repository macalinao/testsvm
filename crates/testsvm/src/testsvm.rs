//! # TestSVM Implementation
//!
//! Core implementation of the TestSVM testing framework wrapper.
//!
//! This module contains the main `TestSVM` struct and its implementation, providing
//! a comprehensive testing environment for Solana programs. It wraps LiteSVM with
//! additional functionality for transaction management, account creation, token operations,
//! and enhanced debugging capabilities.
//!
//! ## Architecture
//!
//! - **LiteSVM Wrapper**: Extends LiteSVM with developer-friendly APIs
//! - **Default Fee Payer**: Automatic transaction fee management
//! - **Address Book**: Integrated labeling system for all accounts
//! - **Transaction Result**: Rich error reporting and transaction analysis
//! - **Token Operations**: Built-in SPL Token program support

use std::{
    env,
    path::{Path, PathBuf},
};

use anchor_spl::token;
use anyhow::*;
use litesvm::LiteSVM;
use solana_sdk::{
    clock::Clock,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::{AccountRef, AddressBook, SeedPart, TXError, TXResult, new_funded_account};

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
            Err(e) => {
                let tx_error = TXError {
                    transaction,
                    metadata: e.clone(),
                };
                tx_error.print_error(&self.address_book);
                self.address_book.print_all();
                Err(Box::new(tx_error))
            }
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

    /// Create a mint with the test SVM's payer and add to address book
    pub fn create_mint(
        &mut self,
        name: &str,
        decimals: u8,
        authority: &Pubkey,
    ) -> Result<AccountRef<anchor_spl::token::Mint>> {
        let mint = Keypair::new();

        let rent = self
            .svm
            .minimum_balance_for_rent_exemption(token::Mint::LEN); // Mint account size

        let create_account_ix = solana_sdk::system_instruction::create_account(
            &self.default_fee_payer.pubkey(),
            &mint.pubkey(),
            rent,
            anchor_spl::token::Mint::LEN as u64, // Mint account size
            &anchor_spl::token::ID,
        );

        let init_mint_ix = anchor_spl::token::spl_token::instruction::initialize_mint(
            &anchor_spl::token::ID,
            &mint.pubkey(),
            authority,
            Some(authority), // Set freeze authority to same as mint authority
            decimals,
        )
        .context("Failed to create initialize mint instruction")?;

        self.execute_ixs_with_signers(&[create_account_ix, init_mint_ix], &[&mint])
            .map_err(|e| anyhow!("Failed to create mint: {}", e))?;

        // Add the mint to the address book
        let mint_pubkey = mint.pubkey();
        let label = format!("mint:{name}");
        self.address_book.add_mint(mint_pubkey, label)?;

        Ok(AccountRef::new(mint_pubkey))
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
    ///
    /// # Arguments
    /// * `fixture_name` - The name of the fixture file (without the .so extension)
    /// * `pubkey` - The public key to assign to the program
    ///
    /// # Example
    /// ```rust,no_run
    /// # use testsvm::TestSVM;
    /// # use solana_sdk::pubkey::Pubkey;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # let mut env = TestSVM::init()?;
    /// # let program_id = Pubkey::new_unique();
    /// // This will load the file from fixtures/programs/my_program.so
    /// env.add_program_fixture("my_program", program_id)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # File Structure
    /// Your project should have the following structure:
    /// ```text
    /// project_root/
    /// ├── fixtures/
    /// │   └── programs/
    /// │       ├── my_program.so
    /// │       └── other_program.so
    /// └── src/
    /// ```
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

    /// Create an associated token account instruction and add to address book
    /// Returns the instruction and the ATA address
    pub fn create_ata_ix(
        &mut self,
        label: &str,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(
        solana_sdk::instruction::Instruction,
        AccountRef<anchor_spl::token::TokenAccount>,
    )> {
        let ata = anchor_spl::associated_token::get_associated_token_address(owner, mint);

        // Add to address book (ignore error if duplicate)
        self.address_book
            .add_ata(ata, label.to_string(), *mint, *owner)?;

        let ix = anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &self.default_fee_payer(),
            owner,
            mint,
            &anchor_spl::token::ID,
        );

        Ok((ix, AccountRef::new(ata)))
    }

    /// Find a PDA with bump and add it to the address book  
    pub fn find_pda_with_bump(
        &mut self,
        label: &str,
        seeds: &[&dyn SeedPart],
        program_id: Pubkey,
    ) -> Result<(Pubkey, u8)> {
        self.address_book
            .find_pda_with_bump(label, seeds, program_id)
    }

    /// Find a PDA and add it to the address book
    pub fn get_pda_key(
        &mut self,
        label: &str,
        seeds: &[&dyn SeedPart],
        program_id: Pubkey,
    ) -> Result<Pubkey> {
        let (pubkey, _bump) = self.find_pda_with_bump(label, seeds, program_id)?;
        Ok(pubkey)
    }

    /// Find a PDA and return an AccountRef with proper type information
    pub fn get_pda<T: anchor_lang::AccountDeserialize>(
        &mut self,
        label: &str,
        seeds: &[&dyn SeedPart],
        program_id: Pubkey,
    ) -> Result<AccountRef<T>> {
        let pubkey = self.get_pda_key(label, seeds, program_id)?;
        Ok(AccountRef::new(pubkey))
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
