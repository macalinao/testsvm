//! # Address Book
//!
//! A comprehensive Solana address management library for tracking and labeling addresses in your applications.
//!
//! This crate provides an organized way to manage Solana public keys with human-readable labels and role-based
//! categorization. It's particularly useful for debugging, transaction analysis, and building developer tools
//! that need to display meaningful information about addresses.
//!
//! ## Features
//!
//! - **Role-based categorization**: Organize addresses by their purpose (wallet, mint, ATA, PDA, program, custom)
//! - **Colored terminal output**: Enhanced readability with color-coded address types
//! - **PDA management**: Built-in support for Program Derived Addresses with seed tracking
//! - **Text replacement**: Automatically replace raw pubkeys with labels in logs and output
//! - **Duplicate prevention**: Ensures label uniqueness across your address book
//!
//! ## Quick Start
//!
//! ```rust
//! use solana_address_book::{AddressBook, RegisteredAddress};
//! use anchor_lang::prelude::*;
//!
//! // Create a new address book
//! let mut book = AddressBook::new();
//!
//! // Add default Solana programs
//! book.add_default_accounts().unwrap();
//!
//! // Add a user wallet
//! let user = Pubkey::new_unique();
//! book.add_wallet(user, "alice_wallet".to_string()).unwrap();
//!
//! // Add a token mint
//! let token_mint = Pubkey::new_unique();
//! book.add(token_mint, "usdc_mint".to_string(), RegisteredAddress::mint(token_mint)).unwrap();
//!
//! // Get a formatted label for display
//! println!("User address: {}", book.format_address(&user));
//! ```
//!
//! ## Usage Examples
//!
//! ### Managing Different Address Types
//!
//! ```rust
//! use solana_address_book::{AddressBook, RegisteredAddress};
//! use anchor_lang::prelude::*;
//!
//! let mut book = AddressBook::new();
//!
//! // Add a wallet address
//! let wallet = Pubkey::new_unique();
//! book.add_wallet(wallet, "treasury".to_string()).unwrap();
//!
//! // Add an Associated Token Account (ATA)
//! let ata = Pubkey::new_unique();
//! let mint = Pubkey::new_unique();
//! let owner = Pubkey::new_unique();
//! book.add(ata, "alice_usdc_ata".to_string(), RegisteredAddress::ata(ata, mint, owner)).unwrap();
//!
//! // Add a Program Derived Address (PDA)
//! let pda = Pubkey::new_unique();
//! let program_id = Pubkey::new_unique();
//! book.add_pda(
//!     pda,
//!     "vault_pda".to_string(),
//!     vec!["vault".to_string(), "seed".to_string()],
//!     program_id,
//!     255
//! ).unwrap();
//!
//! // Add a custom role
//! book.add_custom(
//!     Pubkey::new_unique(),
//!     "governance".to_string(),
//!     "dao_treasury".to_string()
//! ).unwrap();
//! ```
//!
//! ### Finding and Querying Addresses
//!
//! ```rust
//! use solana_address_book::{AddressBook, AddressRole};
//! use anchor_lang::prelude::*;
//!
//! let mut book = AddressBook::new();
//! let wallet = Pubkey::new_unique();
//! book.add_wallet(wallet, "alice".to_string()).unwrap();
//!
//! // Check if an address exists
//! if book.contains(&wallet) {
//!     println!("Address is registered");
//! }
//!
//! // Get label for an address
//! let label = book.get_label(&wallet);
//! println!("Address label: {}", label);
//!
//! // Find address by role
//! if let Some(addr) = book.get_by_role(&AddressRole::Wallet) {
//!     println!("Found wallet: {}", addr);
//! }
//!
//! // Get all addresses of a specific type
//! let all_wallets = book.get_all_by_role_type("wallet");
//! println!("Total wallets: {}", all_wallets.len());
//! ```
//!
//! ### PDA Creation and Registration
//!
//! ```rust
//! use solana_address_book::{AddressBook, RegisteredAddress};
//! use anchor_lang::prelude::*;
//!
//! let mut book = AddressBook::new();
//! let program_id = Pubkey::new_unique();
//!
//! // Create and register a PDA in one step
//! let user = Pubkey::new_unique();
//! let (pda_key, bump) = book.find_pda_with_bump(
//!     "user_vault",
//!     &[b"vault", user.as_ref()],
//!     program_id
//! ).unwrap();
//!
//! // Or create a PDA manually
//! let (pubkey, bump, registered) = RegisteredAddress::pda(
//!     &[b"config", b"v1"],
//!     &program_id
//! );
//! book.add(pubkey, "config_account".to_string(), registered).unwrap();
//! ```
//!
//! ### Text Processing and Display
//!
//! ```rust
//! use solana_address_book::{AddressBook, RegisteredAddress};
//! use anchor_lang::prelude::*;
//!
//! let mut book = AddressBook::new();
//! let token = Pubkey::new_unique();
//! book.add(token, "my_token".to_string(), RegisteredAddress::mint(token)).unwrap();
//!
//! // Replace addresses in text with their labels
//! let log = format!("Transfer from {} to {}", Pubkey::new_unique(), token);
//! let formatted = book.replace_addresses_in_text(&log);
//! println!("{}", formatted); // Will show colored "my_token" instead of raw pubkey
//!
//! // Print entire address book with formatting
//! book.print_all();
//! ```
//!
//! ## Integration with Testing Frameworks
//!
//! This crate is designed to work seamlessly with Solana testing frameworks:
//!
//! ```rust
//! use solana_address_book::{AddressBook, RegisteredAddress};
//! use anchor_lang::prelude::*;
//!
//! fn setup_test_environment() -> AddressBook {
//!     let mut book = AddressBook::new();
//!     
//!     // Add standard programs
//!     book.add_default_accounts().unwrap();
//!     
//!     // Add test accounts
//!     let admin = Pubkey::new_unique();
//!     book.add_wallet(admin, "test_admin".to_string()).unwrap();
//!     
//!     // Track all test tokens
//!     let test_token = Pubkey::new_unique();
//!     book.add(test_token, "test_token".to_string(), RegisteredAddress::mint(test_token))
//!         .unwrap();
//!     
//!     book
//! }
//! ```

pub mod address_book;
pub mod pda_seeds;
pub mod registered_address;

pub use address_book::AddressBook;
pub use pda_seeds::{DerivedPda, find_pda_with_bump_and_strings, seed_to_string};
pub use registered_address::{AddressRole, RegisteredAddress};
