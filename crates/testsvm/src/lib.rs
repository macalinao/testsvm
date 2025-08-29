//! # TestSVM
//!
//! A comprehensive testing framework for Solana SVM (Solana Virtual Machine) programs.
//!
//! This crate provides a developer-friendly wrapper around LiteSVM, offering enhanced debugging,
//! transaction management, and testing utilities for Solana program development.
//!
//! ## Features
//!
//! - **Enhanced LiteSVM Interface**: Simplified API for common testing operations
//! - **Transaction Result Management**: Detailed error reporting and transaction analysis
//! - **Address Book Integration**: Built-in address tracking and labeling
//! - **Account References**: Type-safe account management with automatic tracking
//! - **Colored Output**: Enhanced debugging with color-coded transaction logs
//! - **Helper Functions**: Utilities for airdrop, account creation, and more
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use testsvm::TestSVM;
//! use solana_sdk::pubkey::Pubkey;
//! use solana_sdk::transaction::Transaction;
//! use solana_sdk::signature::Signer;
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//!
//! // Create a new test environment
//! let mut env = TestSVM::init()?;
//!
//! // Add a program to test
//! let program_id = Pubkey::new_unique();
//! env.add_program_from_path(
//!     "my_program",
//!     program_id,
//!     "path/to/program.so"
//! )?;
//!
//! // Create and fund test accounts  
//! let user = env.new_wallet("alice")?;
//!
//! // Build and execute transactions
//! let instructions = vec![
//!     // Your instructions here
//! ];
//! let transaction = Transaction::new_signed_with_payer(
//!     &instructions,
//!     Some(&env.default_fee_payer()),
//!     &[&env.default_fee_payer],
//!     env.svm.latest_blockhash(),
//! );
//! let result = env.execute_transaction(transaction)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Programs
//!
//! ```rust,no_run
//! use testsvm::TestSVM;
//! use solana_sdk::pubkey::Pubkey;
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//!
//! let mut env = TestSVM::init()?;
//!
//! // Load program from file
//! let program_id = Pubkey::new_unique();
//! env.add_program_from_path(
//!     "token_program",
//!     program_id,
//!     "./fixtures/programs/token.so"
//! )?;
//!
//! // Add program fixture from fixtures directory
//! let fixture_program_id = Pubkey::new_unique();
//! env.add_program_fixture("my_program", fixture_program_id)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Account Management
//!
//! ```rust
//! use testsvm::{TestSVM, AccountRef};
//! use solana_sdk::signature::Signer;
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//!
//! let mut env = TestSVM::init()?;
//!
//! // Create wallets with automatic tracking
//! let alice = env.new_wallet("alice")?;
//! let bob = env.new_wallet("bob")?;
//!
//! // Create token mint
//! let mint = env.create_mint("usdc_mint", 6, &alice.pubkey())?;
//!
//! // Create Associated Token Accounts
//! let (alice_ata_ix, alice_ata) = env.create_ata_ix("alice_usdc", &alice.pubkey(), &mint.key)?;
//! let (bob_ata_ix, bob_ata) = env.create_ata_ix("bob_usdc", &bob.pubkey(), &mint.key)?;
//!
//! // Execute the instructions to create the ATAs
//! env.execute_ixs(&[alice_ata_ix, bob_ata_ix])?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Transaction Building and Execution
//!
//! ```rust
//! use testsvm::{TestSVM, TXResultAssertions};
//! use solana_sdk::transaction::Transaction;
//! use solana_sdk::signature::Signer;
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//!
//! let mut env = TestSVM::init()?;
//! let payer = env.new_wallet("payer")?;
//!
//! // Build transaction
//! let instructions = vec![
//!     // Your instructions here
//! ];
//!
//! let tx = Transaction::new_signed_with_payer(
//!     &instructions,
//!     Some(&payer.pubkey()),
//!     &[&payer],
//!     env.svm.latest_blockhash(),
//! );
//!
//! // Execute and verify
//! let result = env.execute_transaction(tx)?;
//!
//! // Access detailed results
//! println!("Compute units used: {}", result.compute_units_consumed);
//! println!("Logs: {:?}", result.logs);
//! # Ok(())
//! # }
//! ```
//!
//! ## Debugging and Analysis
//!
//! ```rust
//! use testsvm::TestSVM;
//! use solana_sdk::transaction::Transaction;
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//!
//! let mut env = TestSVM::init()?;
//!
//! // Execute transaction (example transaction)
//! let instructions = vec![];
//! let tx = Transaction::new_signed_with_payer(
//!     &instructions,
//!     Some(&env.default_fee_payer()),
//!     &[&env.default_fee_payer],
//!     env.svm.latest_blockhash(),
//! );
//! let result = env.execute_transaction(tx)?;
//!
//! // Print formatted output
//! println!("Transaction logs: {:?}", result.logs);
//!
//! // Access address book for debugging
//! env.address_book.print_all();
//!
//! // Get account balance
//! let account = env.default_fee_payer();
//! let account_info = env.svm.get_account(&account);
//! if let Some(info) = account_info {
//!     println!("Account balance: {} lamports", info.lamports);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Integration with Anchor
//!
//! ```rust,no_run
//! use testsvm::{TestSVM, TXResultAssertions};
//! use anchor_lang::prelude::*;
//! use testsvm::anchor_instruction;
//! # use anyhow::Result;
//!
//! // Example program module (would be generated by Anchor)
//! // declare_program!(my_program) would generate something similar to:
//! # use anchor_lang::prelude::*;
//! # use solana_sdk::pubkey::Pubkey;
//! # pub mod my_program {
//! #     use solana_sdk::pubkey::Pubkey;
//! #     pub const ID: Pubkey = Pubkey::new_from_array([0; 32]);
//! #     pub mod accounts {
//! #         use anchor_lang::prelude::*;
//! #         pub struct Initialize {}
//! #         impl anchor_lang::ToAccountMetas for Initialize {
//! #             fn to_account_metas(&self, _: Option<bool>) -> Vec<solana_sdk::instruction::AccountMeta> { vec![] }
//! #         }
//! #     }
//! #     pub mod instruction {
//! #         use anchor_lang::prelude::*;
//! #         #[derive(AnchorSerialize, AnchorDeserialize)]
//! #         pub struct Initialize {}
//! #         impl anchor_lang::Discriminator for Initialize {
//! #             const DISCRIMINATOR: &'static [u8] = &[0; 8];
//! #         }
//! #         impl anchor_lang::InstructionData for Initialize {
//! #             fn data(&self) -> Vec<u8> {
//! #                 let mut data = Vec::with_capacity(8);
//! #                 data.extend_from_slice(&Self::DISCRIMINATOR);
//! #                 data.extend_from_slice(&AnchorSerialize::try_to_vec(self).unwrap_or_default());
//! #                 data
//! #             }
//! #         }
//! #     }
//! # }
//!
//! # fn main() -> Result<()> {
//! let mut env = TestSVM::init()?;
//! let payer = env.new_wallet("payer")?;
//!
//! // Create instruction using Anchor's generated types
//! let ix = anchor_instruction(
//!     my_program::ID,
//!     my_program::accounts::Initialize {},
//!     my_program::instruction::Initialize {},
//! );
//!
//! // Execute in test environment
//! let result = env.execute_ixs_with_signers(&[ix], &[&payer])?;
//! # Ok(())
//! # }
//! ```

pub mod account_ref;
pub mod assertions;
pub mod litesvm_helpers;
pub mod prelude;
pub mod testsvm;
pub mod tx_result;

pub use ::anchor_utils::*;
pub use ::solana_address_book::*;
pub use account_ref::*;
pub use assertions::*;
pub use litesvm_helpers::*;
pub use testsvm::*;
pub use tx_result::*;
