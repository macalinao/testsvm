//! # Account References
//!
//! Type-safe account references for TestSVM with automatic deserialization support.
//!
//! This module provides the `AccountRef` type, which acts as a lightweight handle to
//! on-chain accounts with built-in type safety and Anchor deserialization. It simplifies
//! working with strongly-typed accounts in tests by providing convenient methods for
//! loading and verifying account state.
//!
//! ## Key Features
//!
//! - **Type Safety**: Generic over Anchor account types for compile-time safety
//! - **Loading**: Simple access to account state
//! - **Address Book Integration**: Automatic labeling for better debugging

use crate::TestSVM;
use anchor_lang::Key;
use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::fmt;
use std::marker::PhantomData;

/// A reference to an account on-chain.
#[derive(Copy, Clone, Debug)]
pub struct AccountRef<T: anchor_lang::AccountDeserialize> {
    pub key: Pubkey,
    _phantom: PhantomData<T>,
}

impl<T: anchor_lang::AccountDeserialize> Key for AccountRef<T> {
    fn key(&self) -> Pubkey {
        self.key
    }
}

impl<T: anchor_lang::AccountDeserialize> From<AccountRef<T>> for Pubkey {
    fn from(val: AccountRef<T>) -> Self {
        val.key
    }
}

impl<T: anchor_lang::AccountDeserialize> From<&AccountRef<T>> for Pubkey {
    fn from(val: &AccountRef<T>) -> Self {
        val.key
    }
}

impl<T: anchor_lang::AccountDeserialize> AccountRef<T> {
    /// Create a new account reference
    pub fn new(key: Pubkey) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }

    /// Loads the latest account state from the chain, failing if account doesn't exist
    pub fn load(&self, env: &TestSVM) -> Result<T> {
        self.maybe_load(env)?
            .with_context(|| format!("Account not found: {}", self.key))
    }

    /// Attempts to load the latest account state from the chain, returning None if account doesn't exist
    pub fn maybe_load(&self, env: &TestSVM) -> Result<Option<T>> {
        match env.svm.get_account(&self.key) {
            Some(account) => {
                let mut data = &account.data[..];
                Ok(Some(
                    T::try_deserialize(&mut data).context("Failed to deserialize account")?,
                ))
            }
            None => Ok(None),
        }
    }
}

impl<T: anchor_lang::AccountDeserialize> fmt::Display for AccountRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl<T: anchor_lang::AccountDeserialize> AsRef<[u8]> for AccountRef<T> {
    fn as_ref(&self) -> &[u8] {
        self.key.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::AccountRef;
    use address_book::pda_seeds::{SeedPart, find_pda_with_bump, find_pda_with_bump_and_strings};
    use anchor_lang::prelude::*;

    // Dummy type for testing
    #[derive(Debug, Clone)]
    struct DummyAccount;

    impl anchor_lang::AccountDeserialize for DummyAccount {
        fn try_deserialize_unchecked(_buf: &mut &[u8]) -> Result<Self> {
            Ok(DummyAccount)
        }
    }

    impl anchor_lang::AccountSerialize for DummyAccount {
        fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> Result<()> {
            Ok(())
        }
    }

    impl anchor_lang::Owner for DummyAccount {
        fn owner() -> Pubkey {
            Pubkey::default()
        }
    }

    #[test]
    fn test_account_ref_as_pda_seed() {
        let program_id = Pubkey::new_unique();
        let account_pubkey = Pubkey::new_unique();
        let account_ref: AccountRef<DummyAccount> = AccountRef::new(account_pubkey);

        // Test that AccountRef can be used as a seed
        let seeds: Vec<&dyn SeedPart> = vec![&"prefix", &account_ref];

        let (pda, bump) = find_pda_with_bump(&seeds, &program_id);

        // Verify it matches manual calculation
        let (expected_pda, expected_bump) =
            Pubkey::find_program_address(&[b"prefix", account_pubkey.as_ref()], &program_id);

        assert_eq!(pda, expected_pda);
        assert_eq!(bump, expected_bump);

        // Test with find_pda_with_bump_and_strings
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        assert_eq!(derived_pda.key, expected_pda);
        assert_eq!(derived_pda.bump, expected_bump);

        // Verify string representation
        assert_eq!(derived_pda.seed_strings[0], "prefix");
        assert_eq!(derived_pda.seed_strings[1], account_pubkey.to_string());

        // Verify raw seeds
        assert_eq!(derived_pda.seeds[0], b"prefix");
        assert_eq!(derived_pda.seeds[1], account_pubkey.as_ref());
    }

    #[test]
    fn test_account_ref_mixed_seeds() {
        let program_id = Pubkey::new_unique();
        let vault_account = AccountRef::<DummyAccount>::new(Pubkey::new_unique());
        let owner_account = AccountRef::<DummyAccount>::new(Pubkey::new_unique());
        let nonce: u64 = 42;
        let nonce_bytes = nonce.to_le_bytes();

        // Use multiple AccountRefs in PDA derivation
        let seeds: Vec<&dyn SeedPart> =
            vec![&"vault", &vault_account, &owner_account, &nonce_bytes];

        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Manual verification
        let (expected_pda, expected_bump) = Pubkey::find_program_address(
            &[
                b"vault",
                vault_account.key.as_ref(),
                owner_account.key.as_ref(),
                &nonce_bytes,
            ],
            &program_id,
        );

        assert_eq!(derived_pda.key, expected_pda);
        assert_eq!(derived_pda.bump, expected_bump);
        assert!(derived_pda.verify(&program_id));
    }
}
