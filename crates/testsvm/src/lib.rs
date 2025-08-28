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
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use anchor_lang::prelude::*;
//!
//! // Create a new test environment
//! let mut env = TestSVM::new();
//!
//! // Add a program to test
//! env.add_program_from_file(
//!     "my_program",
//!     &Pubkey::new_unique(),
//!     "path/to/program.so"
//! ).unwrap();
//!
//! // Create and fund test accounts
//! let user = env.create_wallet("alice").unwrap();
//! env.airdrop(&user, 10_000_000_000).unwrap(); // 10 SOL
//!
//! // Execute transactions
//! let result = env.execute_transaction(&transaction)?;
//! result.assert_success()?;
//! ```
//!
//! ## Working with Programs
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use solana_sdk::pubkey::Pubkey;
//!
//! let mut env = TestSVM::new();
//!
//! // Load program from file
//! let program_id = Pubkey::new_unique();
//! env.add_program_from_file(
//!     "token_program",
//!     &program_id,
//!     "./fixtures/programs/token.so"
//! )?;
//!
//! // Deploy program bytecode directly
//! let bytecode = include_bytes!("../programs/my_program.so");
//! env.add_program("my_program", &program_id, bytecode)?;
//! ```
//!
//! ## Account Management
//!
//! ```rust,ignore
//! use testsvm::{TestSVM, AccountRef};
//! use anchor_spl::token;
//!
//! let mut env = TestSVM::new();
//!
//! // Create wallets with automatic tracking
//! let alice = env.create_wallet("alice")?;
//! let bob = env.create_wallet("bob")?;
//!
//! // Create token mint
//! let mint = env.create_mint("usdc_mint", 6)?;
//!
//! // Create Associated Token Accounts
//! let alice_ata = env.create_ata("alice_usdc", &mint, &alice)?;
//! let bob_ata = env.create_ata("bob_usdc", &mint, &bob)?;
//!
//! // Mint tokens
//! env.mint_to(&mint, &alice_ata, 1_000_000)?;
//! ```
//!
//! ## Transaction Building and Execution
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use solana_sdk::transaction::Transaction;
//! use solana_sdk::instruction::Instruction;
//!
//! let mut env = TestSVM::new();
//! let payer = env.create_wallet("payer")?;
//!
//! // Build transaction
//! let instructions = vec![
//!     // Your instructions here
//! ];
//!
//! let tx = Transaction::new_signed_with_payer(
//!     &instructions,
//!     Some(&payer.pubkey()),
//!     &[&payer.signer()],
//!     env.latest_blockhash(),
//! );
//!
//! // Execute and verify
//! let result = env.execute_transaction(&tx)?;
//! result.assert_success()?;
//!
//! // Access detailed results
//! println!("Compute units used: {}", result.compute_units_consumed);
//! println!("Logs: {:?}", result.logs);
//! ```
//!
//! ## Debugging and Analysis
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//!
//! let mut env = TestSVM::new();
//!
//! // Enable detailed logging
//! env.set_compute_max_units(1_000_000);
//!
//! // Execute transaction
//! let result = env.execute_transaction(&tx)?;
//!
//! // Print formatted output
//! result.print_logs();
//!
//! // Access address book for debugging
//! env.address_book.print_all();
//!
//! // Get account balance
//! let balance = env.get_balance(&account)?;
//! println!("Account balance: {} lamports", balance);
//! ```
//!
//! ## Integration with Anchor
//!
//! ```rust,ignore
//! use testsvm::TestSVM;
//! use anchor_lang::prelude::*;
//! use anchor_utils::anchor_instruction;
//!
//! declare_program!(my_program);
//!
//! let mut env = TestSVM::new();
//!
//! // Create instruction using Anchor's generated types
//! let ix = anchor_instruction(
//!     my_program::ID,
//!     my_program::accounts::Initialize {
//!         // Account structs
//!     },
//!     my_program::instruction::Initialize {
//!         // Instruction data
//!     },
//! );
//!
//! // Execute in test environment
//! let result = env.execute_instruction(&ix, &payer)?;
//! result.assert_success()?;
//! ```

pub mod account_ref;
pub mod litesvm_helpers;
pub mod testsvm;
pub mod tx_result;

pub use ::address_book::*;
pub use ::anchor_utils::*;
pub use account_ref::*;
pub use litesvm_helpers::*;
pub use testsvm::*;
pub use tx_result::*;
