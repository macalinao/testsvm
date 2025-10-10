//! # Transaction Assertions
//!
//! Assertion helpers for testing transaction results in TestSVM.
//!
//! This module provides traits and types for asserting expected transaction outcomes,
//! including methods for verifying that transactions succeed or fail with specific errors.
//! These assertions are particularly useful in test environments where you need to
//! verify that your program behaves correctly under various conditions.
//!
//! ## Features
//!
//! - **Success/Failure Assertions**: Verify transactions succeed or fail as expected
//! - **Error Matching**: Check for specific error types including Anchor errors
//! - **Type-safe API**: Compile-time guarantees for assertion chains

use anyhow::*;
use litesvm::types::TransactionMetadata;
use solana_sdk::{instruction::InstructionError, program_error::ProgramError};
use testsvm_core::prelude::*;

/// Provides assertion methods for failed transactions.
///
/// This struct wraps a transaction error and provides helper methods
/// for asserting specific error conditions in tests.
pub struct TXErrorAssertions {
    /// Underlying transaction error.
    pub(crate) error: TXError,
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

    /// Asserts that the transaction failed with a specific error message.
    ///
    /// This method checks the transaction logs for an error message containing
    /// the specified error name and error code.
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

    /// Asserts that the transaction failed with a specific custom error code.
    ///
    /// This is useful for checking SPL Token errors and other program-specific error codes.
    pub fn with_custom_error(&self, error_code: u32) -> Result<()> {
        match self.error.metadata.err.clone() {
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                InstructionError::Custom(code),
            ) => {
                if code == error_code {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Expected custom error code {}, got {}",
                        error_code,
                        code
                    ))
                }
            }
            _ => Err(anyhow!(
                "Expected custom error code {}, but got '{}'",
                error_code,
                self.error.metadata.err.to_string()
            )),
        }
    }

    /// Asserts that the transaction failed with a specific program error.
    pub fn with_program_error<T: Into<ProgramError>>(&self, err: T) -> Result<()> {
        let program_error: ProgramError = err.into();
        match self.error.metadata.err.clone() {
            solana_sdk::transaction::TransactionError::InstructionError(_, err) => {
                let result_program_error: ProgramError = err.try_into()?;
                if result_program_error == program_error {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Expected custom program error {}, but got '{}'",
                        program_error,
                        result_program_error
                    ))
                }
            }
            _ => Err(anyhow!(
                "Expected custom program error {}, but got instruction error '{}'",
                program_error,
                self.error.metadata.err.to_string()
            )),
        }
    }

    /// Returns the underlying transaction error for custom assertions.
    pub fn error(&self) -> &TXError {
        &self.error
    }
}

/// Assertions for successful transactions.
pub struct TXSuccessAssertions {
    /// The successful transaction metadata
    pub metadata: TransactionMetadata,
}

impl TXSuccessAssertions {
    /// Returns the compute units consumed by the transaction.
    pub fn compute_units(&self) -> u64 {
        self.metadata.compute_units_consumed
    }

    /// Returns the transaction logs.
    pub fn logs(&self) -> &Vec<String> {
        &self.metadata.logs
    }
}

/// Extension trait for transaction results providing assertion methods.
///
/// This trait adds convenient assertion methods to `TXResult` for testing
/// whether transactions succeed or fail as expected.
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
    /// ```rust
    /// # use testsvm_core::prelude::*;
    /// # use testsvm_spl::prelude::*;
    /// # use testsvm_assertions::*;
    /// # use solana_sdk::signature::Signer;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # let mut env = TestSVM::init()?;
    /// # let owner = env.new_wallet("owner")?;
    /// # let unauthorized_user = env.new_wallet("unauthorized")?;
    /// #
    /// # // Create a mint owned by 'owner'
    /// # let mint = env.create_mint("test_mint", 6, &owner.pubkey())?;
    /// #
    /// # // Create token accounts
    /// # let (owner_ata_ix, owner_ata) = env.create_ata_ix("owner_ata", &owner.pubkey(), &mint.key)?;
    /// # let (user_ata_ix, user_ata) = env.create_ata_ix("user_ata", &unauthorized_user.pubkey(), &mint.key)?;
    /// # env.execute_ixs(&[owner_ata_ix, user_ata_ix])?;
    /// #
    /// // Try to mint tokens from unauthorized user (should fail)
    /// let mint_ix = anchor_spl::token::spl_token::instruction::mint_to(
    ///     &anchor_spl::token::ID,
    ///     &mint.key,
    ///     &user_ata.key,
    ///     &unauthorized_user.pubkey(), // Wrong authority!
    ///     &[],
    ///     1_000_000,
    /// )?;
    ///
    /// // Test that unauthorized minting fails
    /// let result = env.execute_ixs_with_signers(&[mint_ix], &[&unauthorized_user]);
    /// result
    ///     .fails()?  // Assert the transaction fails
    ///     .with_custom_error(4)?;  // SPL Token error: OwnerMismatch
    /// # Ok(())
    /// # }
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
    /// ```rust
    /// # use testsvm_core::prelude::*;
    /// # use testsvm_spl::prelude::*;
    /// # use testsvm_assertions::*;
    /// # use solana_sdk::signature::Signer;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # let mut env = TestSVM::init()?;
    /// # let owner = env.new_wallet("owner")?;
    /// #
    /// # // Create a mint owned by 'owner'
    /// # let mint = env.create_mint("test_mint", 6, &owner.pubkey())?;
    /// #
    /// # // Create token account
    /// # let (owner_ata_ix, owner_ata) = env.create_ata_ix("owner_ata", &owner.pubkey(), &mint.key)?;
    /// # env.execute_ixs(&[owner_ata_ix])?;
    /// #
    /// // Mint tokens from the authorized owner (should succeed)
    /// let mint_ix = anchor_spl::token::spl_token::instruction::mint_to(
    ///     &anchor_spl::token::ID,
    ///     &mint.key,
    ///     &owner_ata.key,
    ///     &owner.pubkey(), // Correct authority
    ///     &[],
    ///     1_000_000,
    /// )?;
    ///
    /// // Test that authorized minting succeeds
    /// let result = env.execute_ixs_with_signers(&[mint_ix], &[&owner]);
    /// let assertions = result.succeeds()?;
    ///
    /// // Can access transaction metadata
    /// println!("Used {} compute units", assertions.compute_units());
    /// # Ok(())
    /// # }
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
        match self {
            Result::Ok(metadata) => Ok(TXSuccessAssertions { metadata }),
            Result::Err(e) => {
                e.print_error();
                e.address_book.print_all();
                Err(anyhow::anyhow!("Unexpected failed transaction: {}", e))
            }
        }
    }
}
