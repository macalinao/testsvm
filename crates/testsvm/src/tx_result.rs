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

use anyhow::*;
use colored::Colorize;
use litesvm::types::{FailedTransactionMetadata, TransactionMetadata};
use solana_sdk::{instruction::InstructionError, transaction::Transaction};

use crate::AddressBook;

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

pub struct TXErrorAssertions {
    error: TXError,
}

impl TXErrorAssertions {
    /// Asserts that the transaction failed with a specific Anchor error.
    ///
    /// This uses string matching to find the error in the transaction logs, looking for
    /// the last program log containing the string "AnchorError" and matching the error name.
    pub fn with_anchor_error(&self, error_name: &str) -> Result<()> {
        match self.error.metadata.err.clone() {
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                InstructionError::Custom(_error_code),
            ) => {
                let maybe_error_message = self
                    .error
                    .metadata
                    .meta
                    .logs
                    .iter()
                    .rev()
                    .find(|line| line.contains("AnchorError"));
                if let Some(error_message) = maybe_error_message {
                    if error_message.contains(&format!("{error_name}. Error Number:")) {
                        Ok(())
                    } else {
                        Err(anyhow!(
                            "Expected Anchor error '{}', got '{}'",
                            error_name,
                            error_message
                        ))
                    }
                } else {
                    Err(anyhow!(
                        "Expected Anchor error '{}', but nothing was found in the logs",
                        error_name
                    ))
                }
            }
            _ => Err(anyhow!(
                "Expected error containing '{}', but got '{}'",
                error_name,
                self.error.metadata.err.to_string()
            )),
        }
    }

    pub fn with_error(&self, error_name: &str) -> Result<()> {
        match self.error.metadata.err.clone() {
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                InstructionError::Custom(error_code),
            ) => {
                if self
                    .error
                    .metadata
                    .meta
                    .pretty_logs()
                    .contains(format!("{error_name}. Error Number: {error_code}").as_str())
                {
                    Ok(())
                } else {
                    Err(anyhow!("Expected error '{}'", error_name))
                }
            }
            _ => Err(anyhow!(
                "Expected error containing '{}', but got '{}'",
                error_name,
                self.error.metadata.err.to_string()
            )),
        }
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

/// Assertions for successful transactions.
pub struct TXSuccessAssertions {
    pub metadata: TransactionMetadata,
}

pub trait TXResultAssertions {
    /// Asserts that the transaction fails, converting a successful transaction to an error.
    ///
    /// This method is used in tests to verify that a transaction is expected to fail.
    /// It returns a `TXErrorAssertions` struct that provides additional assertion methods
    /// for checking specific error conditions.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TXErrorAssertions)` if the transaction failed as expected, or an error
    /// if the transaction unexpectedly succeeded.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Test that an unauthorized user cannot perform an action
    /// rewarder
    ///     .set_annual_rewards_rate(&mut env, 1_000_000, &unauthorized_user)
    ///     .fails()?  // Assert the transaction fails
    ///     .with_anchor_error("Unauthorized")?;  // Assert it fails with specific error
    /// ```
    fn fails(self) -> Result<TXErrorAssertions>;

    /// Asserts that the transaction succeeds, converting a failed transaction to an error.
    ///
    /// This method is used in tests to verify that a transaction is expected to succeed.
    /// It returns a `TXSuccessAssertions` struct that contains the successful transaction
    /// metadata for further validation if needed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TXSuccessAssertions)` if the transaction succeeded as expected, or an error
    /// if the transaction unexpectedly failed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Test that an authorized user can perform an action
    /// env.execute_ixs_with_signers(&[accept_ix], &[&new_authority])
    ///     .succeeds()?;  // Assert the transaction succeeds
    /// ```
    fn succeeds(self) -> Result<TXSuccessAssertions>;
}

impl TXResultAssertions for TXResult {
    fn fails(self) -> Result<TXErrorAssertions> {
        let err = self
            .err()
            .ok_or(anyhow::anyhow!("Unexpected successful transaction"))?;
        Ok(TXErrorAssertions { error: *err })
    }

    fn succeeds(self) -> Result<TXSuccessAssertions> {
        let metadata = self?;
        Ok(TXSuccessAssertions { metadata })
    }
}
