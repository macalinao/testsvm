//! # PDA Seeds Management
//!
//! Utilities for working with Program Derived Addresses (PDAs) and their seeds.
//!
//! This module provides types and functions for creating, managing, and debugging
//! PDAs in Solana programs. It includes a flexible seed system that can handle
//! various data types and provides human-readable representations for debugging.
//!
//! ## Features
//!
//! - **Flexible Seed Types**: Support for strings, pubkeys, and raw bytes as seeds
//! - **PDA Derivation**: Helper functions for finding PDAs with bumps
//! - **Debug Support**: Human-readable seed representations for logging
//! - **Verification**: Methods to verify PDA derivation correctness

use anchor_lang::prelude::*;

/// Result of PDA derivation containing all relevant information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedPda {
    /// The derived public key
    pub key: Pubkey,
    /// The bump seed used to derive the PDA
    pub bump: u8,
    /// String representations of seeds for debugging
    pub seed_strings: Vec<String>,
    /// Raw seed bytes used for derivation
    pub seeds: Vec<Vec<u8>>,
}

impl DerivedPda {
    /// Verify that this PDA matches what find_program_address would return
    pub fn verify(&self, program_id: &Pubkey) -> bool {
        let seed_refs: Vec<&[u8]> = self.seeds.iter().map(|s| s.as_slice()).collect();
        let (expected_key, expected_bump) = Pubkey::find_program_address(&seed_refs, program_id);
        self.key == expected_key && self.bump == expected_bump
    }
}

/// Trait for types that can be used as PDA seeds
pub trait SeedPart: AsRef<[u8]> {}

// Blanket implementation for all types that can be referenced as byte slices
impl<T: AsRef<[u8]> + ?Sized> SeedPart for T {}

/// Convert a seed to a string representation for debugging
pub fn seed_to_string(seed: &dyn SeedPart) -> String {
    // Try to convert common types to readable strings
    let bytes = seed.as_ref();

    // Check if it's likely a string (all printable ASCII)
    if bytes.iter().all(|&b| b.is_ascii_graphic() || b == b' ') {
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }
    }

    // Check if it's a pubkey (32 bytes)
    if bytes.len() == 32 {
        if let Ok(pubkey) = Pubkey::try_from(bytes) {
            return pubkey.to_string();
        }
    }

    // Default to hex encoding for other byte arrays
    hex::encode(bytes)
}

/// Find a PDA with bump given seeds and program ID
///
/// This function calculates a Program Derived Address (PDA) and its bump seed
/// from the provided seeds and program ID.
///
/// # Arguments
/// * `seeds` - Array of seed parts that implement the SeedPart trait
/// * `program_id` - The program ID to derive the address for
///
/// # Returns
/// * `(Pubkey, u8)` - The derived public key and bump seed
///
/// # Example
/// ```
/// use anchor_lang::prelude::*;
/// use address_book::pda_seeds::{find_pda_with_bump, SeedPart};
///
/// let program_id = Pubkey::new_unique();
/// let seeds: Vec<&dyn SeedPart> = vec![&"user", &"profile"];
/// let (pda, bump) = find_pda_with_bump(&seeds, &program_id);
/// ```
pub fn find_pda_with_bump(seeds: &[&dyn SeedPart], program_id: &Pubkey) -> (Pubkey, u8) {
    // Convert seeds to byte slices for PDA calculation
    let seed_bytes: Vec<&[u8]> = seeds.iter().map(|s| s.as_ref()).collect();

    // Find the PDA and bump
    Pubkey::find_program_address(&seed_bytes, program_id)
}

/// Find a PDA with bump and return along with seed strings for display
///
/// This function is similar to `find_pda_with_bump` but also returns
/// string representations of the seeds for debugging/display purposes.
///
/// # Arguments
/// * `seeds` - Array of seed parts that implement the SeedPart trait
/// * `program_id` - The program ID to derive the address for
///
/// # Returns
/// * `DerivedPda` - Struct containing the derived public key, bump seed, seed strings, and raw seeds
pub fn find_pda_with_bump_and_strings(seeds: &[&dyn SeedPart], program_id: &Pubkey) -> DerivedPda {
    // Convert seeds to byte slices for PDA calculation
    let seed_bytes: Vec<&[u8]> = seeds.iter().map(|s| s.as_ref()).collect();

    // Find the PDA and bump
    let (pubkey, bump) = Pubkey::find_program_address(&seed_bytes, program_id);

    // Convert seeds to strings for display
    let seed_strings: Vec<String> = seeds.iter().map(|s| seed_to_string(*s)).collect();

    // Store owned copies of the seed bytes
    let seeds_owned: Vec<Vec<u8>> = seed_bytes.iter().map(|s| s.to_vec()).collect();

    DerivedPda {
        key: pubkey,
        bump,
        seed_strings,
        seeds: seeds_owned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_seed() {
        let program_id = Pubkey::new_unique();
        let seed = "test_seed";
        let seeds: Vec<&dyn SeedPart> = vec![&seed];

        let (pda, bump) = find_pda_with_bump(&seeds, &program_id);

        // Verify the PDA is deterministic
        let (pda2, bump2) = find_pda_with_bump(&seeds, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);

        // Verify the PDA is on curve
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_multiple_string_seeds() {
        let program_id = Pubkey::new_unique();
        let seed1 = "user";
        let seed2 = "profile";
        let seeds: Vec<&dyn SeedPart> = vec![&seed1, &seed2];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);

        // Verify different seed order produces different PDA
        let seeds_reversed: Vec<&dyn SeedPart> = vec![&seed2, &seed1];
        let (pda_reversed, _) = find_pda_with_bump(&seeds_reversed, &program_id);
        assert_ne!(pda, pda_reversed);
    }

    #[test]
    fn test_pubkey_seed() {
        let program_id = Pubkey::new_unique();
        let user_pubkey = Pubkey::new_unique();
        let seeds: Vec<&dyn SeedPart> = vec![&user_pubkey];

        let (pda, bump) = find_pda_with_bump(&seeds, &program_id);

        // Verify the PDA is deterministic with pubkey seed
        let (pda2, bump2) = find_pda_with_bump(&seeds, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_mixed_seeds() {
        let program_id = Pubkey::new_unique();
        let prefix = "vault";
        let owner = Pubkey::new_unique();
        let id: u64 = 12345;
        let id_bytes = id.to_le_bytes();

        let seeds: Vec<&dyn SeedPart> = vec![&prefix, &owner, &id_bytes];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_byte_array_seed() {
        let program_id = Pubkey::new_unique();
        let bytes: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let seeds: Vec<&dyn SeedPart> = vec![&bytes];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_slice_seed() {
        let program_id = Pubkey::new_unique();
        let vec_bytes = vec![9, 8, 7, 6, 5, 4, 3, 2, 1];
        let seeds: Vec<&dyn SeedPart> = vec![&vec_bytes];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_find_pda_with_strings() {
        let program_id = Pubkey::new_unique();
        let seed1 = "metadata";
        let seed2 = Pubkey::new_unique();
        let seed3: u32 = 42;
        let seed3_bytes = seed3.to_le_bytes();

        let seeds: Vec<&dyn SeedPart> = vec![&seed1, &seed2, &seed3_bytes];

        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Verify we got the right number of string representations
        assert_eq!(derived_pda.seed_strings.len(), 3);
        assert_eq!(derived_pda.seed_strings[0], "metadata");
        assert_eq!(derived_pda.seed_strings[1], seed2.to_string());

        // Verify the PDA matches what we'd get from the basic function
        let (pda2, bump2) = find_pda_with_bump(&seeds, &program_id);
        assert_eq!(derived_pda.key, pda2);
        assert_eq!(derived_pda.bump, bump2);

        // Verify the stored seeds are correct
        assert_eq!(derived_pda.seeds.len(), 3);
        assert_eq!(derived_pda.seeds[0], seed1.as_bytes());
        assert_eq!(derived_pda.seeds[1], seed2.as_ref());
        assert_eq!(derived_pda.seeds[2], seed3_bytes.as_ref());

        // Verify that find_program_address returns the same result
        let seed_refs: Vec<&[u8]> = derived_pda.seeds.iter().map(|s| s.as_slice()).collect();
        let (expected_key, expected_bump) = Pubkey::find_program_address(&seed_refs, &program_id);
        assert_eq!(derived_pda.key, expected_key);
        assert_eq!(derived_pda.bump, expected_bump);

        // Verify the verify method works
        assert!(derived_pda.verify(&program_id));
    }

    #[test]
    fn test_empty_seeds() {
        let program_id = Pubkey::new_unique();
        let seeds: Vec<&dyn SeedPart> = vec![];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);

        // Empty seeds should still produce a valid PDA
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_max_seed_length() {
        let program_id = Pubkey::new_unique();
        // Max seed length is 32 bytes per seed
        let max_seed = vec![0u8; 32];
        let seeds: Vec<&dyn SeedPart> = vec![&max_seed];

        let (pda, _bump) = find_pda_with_bump(&seeds, &program_id);
        assert!(!pda.is_on_curve());
    }

    #[test]
    fn test_different_programs_same_seeds() {
        let program_id1 = Pubkey::new_unique();
        let program_id2 = Pubkey::new_unique();
        let seed = "same_seed";
        let seeds: Vec<&dyn SeedPart> = vec![&seed];

        let (pda1, _) = find_pda_with_bump(&seeds, &program_id1);
        let (pda2, _) = find_pda_with_bump(&seeds, &program_id2);

        // Same seeds with different programs should produce different PDAs
        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_bump_determinism() {
        let program_id = Pubkey::new_unique();
        let seed = "deterministic";
        let seeds: Vec<&dyn SeedPart> = vec![&seed];

        // Run multiple times to ensure determinism
        let results: Vec<(Pubkey, u8)> = (0..10)
            .map(|_| find_pda_with_bump(&seeds, &program_id))
            .collect();

        // All results should be identical
        for result in &results[1..] {
            assert_eq!(result.0, results[0].0);
            assert_eq!(result.1, results[0].1);
        }
    }

    #[test]
    fn test_known_pda() {
        // Test with a known program ID to ensure consistency
        let program_id = Pubkey::default(); // All zeros
        let seed = "test";
        let seeds: Vec<&dyn SeedPart> = vec![&seed];

        let (pda, bump) = find_pda_with_bump(&seeds, &program_id);

        // The PDA should be consistent for these known inputs
        let (expected_pda, expected_bump) =
            Pubkey::find_program_address(&[seed.as_bytes()], &program_id);

        assert_eq!(pda, expected_pda);
        assert_eq!(bump, expected_bump);
    }

    #[test]
    fn test_derived_pda_verify() {
        let program_id = Pubkey::new_unique();
        let seed1 = "vault";
        let seed2 = Pubkey::new_unique();
        let seeds: Vec<&dyn SeedPart> = vec![&seed1, &seed2];

        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Verify method should return true for correct program_id
        assert!(derived_pda.verify(&program_id));

        // Verify method should return false for different program_id
        let different_program = Pubkey::new_unique();
        assert!(!derived_pda.verify(&different_program));

        // Manually verify that stored seeds produce the same PDA
        let seed_refs: Vec<&[u8]> = derived_pda.seeds.iter().map(|s| s.as_slice()).collect();
        let (manual_key, manual_bump) = Pubkey::find_program_address(&seed_refs, &program_id);
        assert_eq!(derived_pda.key, manual_key);
        assert_eq!(derived_pda.bump, manual_bump);
    }

    #[test]
    fn test_derived_pda_fields() {
        let program_id = Pubkey::new_unique();
        let string_seed = "config";
        let pubkey_seed = Pubkey::new_unique();
        let byte_seed: u64 = 999;
        let byte_seed_bytes = byte_seed.to_le_bytes();

        let seeds: Vec<&dyn SeedPart> = vec![&string_seed, &pubkey_seed, &byte_seed_bytes];
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Check all fields are populated correctly
        assert!(!derived_pda.key.is_on_curve());
        // bump is a u8 so it's always <= 255
        assert_eq!(derived_pda.seed_strings.len(), 3);
        assert_eq!(derived_pda.seeds.len(), 3);

        // Check string representations
        assert_eq!(derived_pda.seed_strings[0], "config");
        assert_eq!(derived_pda.seed_strings[1], pubkey_seed.to_string());
        // Byte arrays are hex encoded
        assert_eq!(derived_pda.seed_strings[2], hex::encode(byte_seed_bytes));

        // Check raw seeds match inputs
        assert_eq!(derived_pda.seeds[0], string_seed.as_bytes());
        assert_eq!(derived_pda.seeds[1], pubkey_seed.as_ref());
        assert_eq!(derived_pda.seeds[2], byte_seed_bytes.as_ref());

        // Verify consistency with find_program_address
        assert!(derived_pda.verify(&program_id));
    }

    #[test]
    fn test_explicit_miner_pda_example() {
        // Example: Miner PDA like in quarry-mine
        let program_id = Pubkey::new_unique(); // Would be quarry_mine::ID in real code
        let replica_quarry = Pubkey::new_unique();
        let merge_miner = Pubkey::new_unique();

        // Manual calculation exactly as in real code
        let (expected_miner_pda, expected_bump) = Pubkey::find_program_address(
            &[b"Miner", replica_quarry.as_ref(), merge_miner.as_ref()],
            &program_id,
        );

        // Using our helper with the same seeds
        let seeds: Vec<&dyn SeedPart> = vec![&"Miner", &replica_quarry, &merge_miner];
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Verify they produce the same result
        assert_eq!(derived_pda.key, expected_miner_pda);
        assert_eq!(derived_pda.bump, expected_bump);

        // Verify seed strings are useful for debugging
        assert_eq!(derived_pda.seed_strings[0], "Miner");
        assert_eq!(derived_pda.seed_strings[1], replica_quarry.to_string());
        assert_eq!(derived_pda.seed_strings[2], merge_miner.to_string());

        // Verify raw seeds match what was passed to find_program_address
        assert_eq!(derived_pda.seeds[0], b"Miner");
        assert_eq!(derived_pda.seeds[1], replica_quarry.as_ref());
        assert_eq!(derived_pda.seeds[2], merge_miner.as_ref());
    }

    #[test]
    fn test_explicit_vault_pda_example() {
        // Example: Token vault PDA pattern
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let token_mint = Pubkey::new_unique();
        let vault_id: u64 = 1;

        // Manual calculation as would be done in production
        let (expected_vault_pda, expected_bump) = Pubkey::find_program_address(
            &[
                b"vault",
                authority.as_ref(),
                token_mint.as_ref(),
                &vault_id.to_le_bytes(),
            ],
            &program_id,
        );

        // Using our helper
        let vault_id_bytes = vault_id.to_le_bytes();
        let seeds: Vec<&dyn SeedPart> = vec![&"vault", &authority, &token_mint, &vault_id_bytes];
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);

        // Must produce identical results
        assert_eq!(derived_pda.key, expected_vault_pda);
        assert_eq!(derived_pda.bump, expected_bump);
        assert!(derived_pda.verify(&program_id));
    }

    #[test]
    fn test_explicit_metadata_pda_example() {
        // Example: Metaplex metadata account pattern
        let metadata_program_id = Pubkey::new_unique(); // Would be mpl_token_metadata::ID
        let mint_pubkey = Pubkey::new_unique();

        // Manual calculation as in Metaplex
        let (expected_metadata_pda, expected_bump) = Pubkey::find_program_address(
            &[
                b"metadata",
                metadata_program_id.as_ref(),
                mint_pubkey.as_ref(),
            ],
            &metadata_program_id,
        );

        // Using our helper
        let seeds: Vec<&dyn SeedPart> = vec![&"metadata", &metadata_program_id, &mint_pubkey];
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &metadata_program_id);

        // Verify exact match
        assert_eq!(derived_pda.key, expected_metadata_pda);
        assert_eq!(derived_pda.bump, expected_bump);

        // Verify seeds are stored correctly
        assert_eq!(derived_pda.seeds[0], b"metadata");
        assert_eq!(derived_pda.seeds[1], metadata_program_id.as_ref());
        assert_eq!(derived_pda.seeds[2], mint_pubkey.as_ref());
    }

    #[test]
    fn test_explicit_associated_token_account_example() {
        // Example: ATA derivation pattern
        let token_program_id = Pubkey::new_unique(); // Would be spl_token::ID
        let associated_token_program_id = Pubkey::new_unique(); // Would be spl_associated_token_account::ID
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        // Manual ATA calculation
        let (expected_ata, _expected_bump) = Pubkey::find_program_address(
            &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
            &associated_token_program_id,
        );

        // Using our helper
        let seeds: Vec<&dyn SeedPart> = vec![&wallet, &token_program_id, &mint];
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &associated_token_program_id);

        // Must match the expected ATA
        assert_eq!(derived_pda.key, expected_ata);

        // Verify we can reconstruct the same PDA from stored seeds
        let (reconstructed_ata, reconstructed_bump) = Pubkey::find_program_address(
            &[
                &derived_pda.seeds[0],
                &derived_pda.seeds[1],
                &derived_pda.seeds[2],
            ],
            &associated_token_program_id,
        );
        assert_eq!(reconstructed_ata, expected_ata);
        assert_eq!(reconstructed_bump, derived_pda.bump);
    }

    #[test]
    fn test_explicit_escrow_pda_example() {
        // Example: Escrow account with mixed seed types
        let program_id = Pubkey::new_unique();
        let initializer = Pubkey::new_unique();
        let escrow_seed = b"escrow";
        let escrow_id: u32 = 12345;
        let timestamp: i64 = 1234567890;

        // Manual calculation with explicit byte arrays
        let (expected_escrow_pda, expected_bump) = Pubkey::find_program_address(
            &[
                escrow_seed,
                initializer.as_ref(),
                &escrow_id.to_le_bytes(),
                &timestamp.to_le_bytes(),
            ],
            &program_id,
        );

        // Using our helper with same seeds
        let escrow_id_bytes = escrow_id.to_le_bytes();
        let timestamp_bytes = timestamp.to_le_bytes();
        let seeds: Vec<&dyn SeedPart> =
            vec![&"escrow", &initializer, &escrow_id_bytes, &timestamp_bytes];

        // First, use the basic function
        let (pda_basic, bump_basic) = find_pda_with_bump(&seeds, &program_id);
        assert_eq!(pda_basic, expected_escrow_pda);
        assert_eq!(bump_basic, expected_bump);

        // Then use the detailed function
        let derived_pda = find_pda_with_bump_and_strings(&seeds, &program_id);
        assert_eq!(derived_pda.key, expected_escrow_pda);
        assert_eq!(derived_pda.bump, expected_bump);

        // Verify stored seeds exactly match what we passed to find_program_address
        assert_eq!(derived_pda.seeds[0], escrow_seed);
        assert_eq!(derived_pda.seeds[1], initializer.as_ref());
        assert_eq!(derived_pda.seeds[2], escrow_id.to_le_bytes().as_ref());
        assert_eq!(derived_pda.seeds[3], timestamp.to_le_bytes().as_ref());

        // Double-check by manually reconstructing
        let (manual_check, manual_bump) = Pubkey::find_program_address(
            &[
                &derived_pda.seeds[0],
                &derived_pda.seeds[1],
                &derived_pda.seeds[2],
                &derived_pda.seeds[3],
            ],
            &program_id,
        );
        assert_eq!(manual_check, expected_escrow_pda);
        assert_eq!(manual_bump, expected_bump);
    }
}
