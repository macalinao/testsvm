//! # TestSVM SPL Helpers
//!
//! SPL Token helper functions for the TestSVM testing framework.
//!
//! This crate provides the `TestSVMSPLHelpers` trait that extends TestSVM
//! with SPL Token-specific functionality for creating mints, token accounts,
//! and performing token operations.

use anchor_spl::token;
use anyhow::{anyhow, Context, Result};
use testsvm_core::prelude::*;

pub mod prelude;

/// SPL Token helper functions for TestSVM
pub trait TestSVMSPLHelpers {
    /// Create a mint with the test SVM's payer and add to address book
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the mint in the address book
    /// * `decimals` - Number of decimals for the token
    /// * `authority` - Mint and freeze authority for the token
    ///
    /// # Example
    ///
    /// ```
    /// use testsvm_core::prelude::*;
    /// use testsvm_spl::TestSVMSPLHelpers;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut svm = TestSVM::init()?;
    /// let authority = Keypair::new();
    ///
    /// // Create a mint with 6 decimals (like USDC)
    /// let mint = svm.create_mint("usdc", 6, &authority.pubkey())?;
    ///
    /// // The mint is automatically added to the address book
    /// assert!(svm.address_book.contains(&mint.key));
    ///
    /// // Read the mint information from chain
    /// let mint_data: anchor_spl::token::Mint = mint.load(&svm)?;
    /// assert_eq!(mint_data.decimals, 6);
    /// assert_eq!(mint_data.mint_authority.unwrap(), authority.pubkey());
    /// assert_eq!(mint_data.supply, 0);
    /// assert!(mint_data.is_initialized);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example - Minting tokens
    ///
    /// ```
    /// use testsvm_core::prelude::*;
    /// use testsvm_spl::TestSVMSPLHelpers;
    /// use anchor_spl::token;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut svm = TestSVM::init()?;
    /// let authority = svm.new_wallet("authority")?;
    /// let user = svm.new_wallet("user")?;
    ///
    /// // Create a mint
    /// let mint = svm.create_mint("my_token", 9, &authority.pubkey())?;
    ///
    /// // Create an ATA for the user
    /// let (create_ata_ix, user_ata) = svm.create_ata_ix(
    ///     "user_ata",
    ///     &user.pubkey(),
    ///     &mint.key
    /// )?;
    /// svm.execute_ixs(&[create_ata_ix])?;
    ///
    /// // Mint 1000 tokens to the user (with 9 decimals)
    /// let mint_amount = 1000 * 10u64.pow(9);
    /// let mint_ix = token::spl_token::instruction::mint_to(
    ///     &token::ID,
    ///     &mint.key,
    ///     &user_ata.key,
    ///     &authority.pubkey(),
    ///     &[],
    ///     mint_amount,
    /// )?;
    /// svm.execute_ixs_with_signers(&[mint_ix], &[&authority])?;
    ///
    /// // Verify the mint supply was updated
    /// let mint_data: token::Mint = mint.load(&svm)?;
    /// assert_eq!(mint_data.supply, mint_amount);
    ///
    /// // Verify the user received the tokens
    /// let ata_data: token::TokenAccount = user_ata.load(&svm)?;
    /// assert_eq!(ata_data.amount, mint_amount);
    /// # Ok(())
    /// # }
    /// ```
    fn create_mint(
        &mut self,
        name: &str,
        decimals: u8,
        authority: &Pubkey,
    ) -> Result<AccountRef<anchor_spl::token::Mint>>;

    /// Create an associated token account instruction and add to address book
    ///
    /// Returns the instruction and the ATA address. The instruction must be executed
    /// separately to actually create the account on-chain.
    ///
    /// # Arguments
    ///
    /// * `label` - Label for the ATA in the address book
    /// * `owner` - Owner of the associated token account
    /// * `mint` - Mint for which to create the ATA
    ///
    /// # Example
    ///
    /// ```
    /// use testsvm_core::prelude::*;
    /// use testsvm_spl::TestSVMSPLHelpers;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut svm = TestSVM::init()?;
    /// let authority = Keypair::new();
    /// let user = Keypair::new();
    ///
    /// // Create a mint first
    /// let mint = svm.create_mint("token", 9, &authority.pubkey())?;
    ///
    /// // Create an ATA instruction for the user
    /// let (ix, ata) = svm.create_ata_ix("user_token_ata", &user.pubkey(), &mint.key)?;
    ///
    /// // Execute the instruction to create the ATA on-chain
    /// svm.execute_ixs(&[ix])?;
    ///
    /// // The ATA is automatically added to the address book
    /// assert!(svm.address_book.contains(&ata.key));
    ///
    /// // Read the ATA information from chain
    /// let ata_data: anchor_spl::token::TokenAccount = ata.load(&svm)?;
    /// assert_eq!(ata_data.owner, user.pubkey());
    /// assert_eq!(ata_data.mint, mint.key);
    /// assert_eq!(ata_data.amount, 0);
    /// assert_eq!(ata_data.state, anchor_spl::token::spl_token::state::AccountState::Initialized);
    /// # Ok(())
    /// # }
    /// ```
    fn create_ata_ix(
        &mut self,
        label: &str,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(
        solana_sdk::instruction::Instruction,
        AccountRef<anchor_spl::token::TokenAccount>,
    )>;
}

impl TestSVMSPLHelpers for TestSVM {
    fn create_mint(
        &mut self,
        name: &str,
        decimals: u8,
        authority: &Pubkey,
    ) -> Result<AccountRef<anchor_spl::token::Mint>> {
        let mint = Keypair::new();

        let rent = self
            .svm
            .minimum_balance_for_rent_exemption(token::Mint::LEN);

        let create_account_ix = solana_sdk::system_instruction::create_account(
            &self.default_fee_payer.pubkey(),
            &mint.pubkey(),
            rent,
            anchor_spl::token::Mint::LEN as u64,
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

        // Add the mint to the address book
        let mint_pubkey = mint.pubkey();
        let label = format!("mint:{name}");
        self.address_book
            .add(mint_pubkey, label, RegisteredAddress::mint(mint_pubkey))?;

        self.execute_ixs_with_signers(&[create_account_ix, init_mint_ix], &[&mint])
            .map_err(|e| anyhow!("Failed to create mint: {}", e))?;

        Ok(AccountRef::new(mint_pubkey))
    }

    fn create_ata_ix(
        &mut self,
        label: &str,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(
        solana_sdk::instruction::Instruction,
        AccountRef<anchor_spl::token::TokenAccount>,
    )> {
        let ata = anchor_spl::associated_token::get_associated_token_address(owner, mint);

        // Add to address book
        self.address_book.add(
            ata,
            label.to_string(),
            RegisteredAddress::ata(ata, *mint, *owner),
        )?;

        let ix = anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &self.default_fee_payer(),
            owner,
            mint,
            &anchor_spl::token::ID,
        );

        Ok((ix, AccountRef::new(ata)))
    }
}
