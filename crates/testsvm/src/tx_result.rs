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
                    if error_message.contains(&format!("{}. Error Number:", error_name)) {
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
                    .contains(format!("{}. Error Number: {}", error_name, error_code).as_str())
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
        // for log in self.metadata.clone().meta.logs {
        //     // Replace addresses in log messages
        //     let formatted_log = address_book.replace_addresses_in_text(&log);
        //     println!("   {}", formatted_log);
        // }

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

pub type TXResult = Result<TransactionMetadata, TXError>;

pub struct TXSuccessAssertions {
    pub metadata: TransactionMetadata,
}

pub trait TXResultHelpers {
    fn fails(self) -> Result<TXErrorAssertions>;

    fn succeeds(self) -> Result<TXSuccessAssertions>;
}

impl TXResultHelpers for TXResult {
    fn fails(self) -> Result<TXErrorAssertions> {
        let err = self
            .err()
            .ok_or(anyhow::anyhow!("Unexpected successful transaction"))?;
        Ok(TXErrorAssertions { error: err })
    }

    fn succeeds(self) -> Result<TXSuccessAssertions> {
        let metadata = self?;
        Ok(TXSuccessAssertions { metadata })
    }
}
