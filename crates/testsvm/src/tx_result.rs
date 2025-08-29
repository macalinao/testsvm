//! # Transaction Result Management
//!
//! Enhanced transaction result handling with detailed error reporting and analysis.
//!
//! This module provides rich transaction result types that extend LiteSVM's basic
//! transaction metadata with additional debugging capabilities, colored output,
//! and assertion helpers. It makes it easy to understand why transactions failed
//! and provides useful information for successful transactions.
//!
//! ## Features
//!
//! - **Detailed Error Information**: Comprehensive error context including logs and instruction errors
//! - **Colored Output**: Enhanced readability with color-coded transaction logs
//! - **Assertion Helpers**: Built-in methods for testing expected outcomes
//! - **Address Resolution**: Automatic replacement of addresses with labels from the address book
//! - **Anchor Error Support**: Special handling for Anchor framework error codes

use std::error::Error;
use std::fmt::Display;

use colored::Colorize;
use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_sdk::transaction::Transaction;

use crate::AddressBook;

/// Error type representing a failed transaction with detailed metadata.
///
/// Contains both the original transaction and the failure metadata from LiteSVM,
/// allowing for comprehensive error analysis and debugging.
#[derive(Debug)]
pub struct TXError {
    /// The transaction that failed
    pub transaction: Transaction,
    /// Underlying failed transaction metadata
    pub metadata: FailedTransactionMetadata,
}

impl Error for TXError {}

impl Display for TXError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction failed: {}", self.metadata.err)
    }
}

impl TXError {
    /// Print the error details, formatted using an [AddressBook].
    pub fn print_error(&self, address_book: &AddressBook) {
        println!(
            "\n{} {}",
            "❌".red(),
            "Transaction failed with error:".red().bold()
        );
        println!("   {}", format!("{:?}", self.metadata.err).bright_red());

        println!(
            "\n{} {}",
            "📜".yellow(),
            "Transaction Logs:".yellow().bold()
        );
        println!(
            "{}",
            address_book.replace_addresses_in_text(&self.metadata.meta.pretty_logs())
        );

        // Log each instruction for debugging
        println!(
            "\n{} {}",
            "📋".blue(),
            "Instructions in failed transaction:".blue().bold()
        );
        for (i, ix) in self.transaction.message.instructions.iter().enumerate() {
            let program_id = self.transaction.message.account_keys[ix.program_id_index as usize];
            println!(
                "   {} {}: {}",
                "Instruction".dimmed(),
                i.to_string().bold(),
                address_book.format_address(&program_id)
            );
            println!(
                "   {} {}",
                "Accounts:".dimmed(),
                format!("{} total", ix.accounts.len()).cyan()
            );

            // Show account details with labels from address book
            for (j, account_index) in ix.accounts.iter().enumerate() {
                let account_key = self.transaction.message.account_keys[*account_index as usize];
                let is_signer = self.transaction.message.is_signer(*account_index as usize);
                let is_writable = self
                    .transaction
                    .message
                    .is_maybe_writable(*account_index as usize, None);

                let mut flags = Vec::new();
                if is_signer {
                    flags.push("signer".green().to_string());
                }
                if is_writable {
                    flags.push("writable".yellow().to_string());
                }

                let flags_str = if !flags.is_empty() {
                    format!(" [{}]", flags.join(", "))
                } else {
                    String::new()
                };

                println!(
                    "     {} {}: {}{}",
                    "Account".dimmed(),
                    j.to_string().bold(),
                    address_book.format_address(&account_key),
                    flags_str
                );
            }
        }

        // Print the address book for reference
    }
}

/// A result type that represents the result of a transaction.
pub type TXResult = Result<TransactionMetadata, Box<TXError>>;
