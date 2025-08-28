//! # Merge Pool Testing Utilities
//!
//! Test helpers for Quarry merge pool management.
//!
//! This module provides the `TestMergePool` struct for creating and managing
//! merge pools in the Quarry protocol. Merge pools enable multiple quarries
//! to share rewards through a unified staking mechanism, where miners can
//! earn from both primary and replica token pools.
//!
//! ## Features
//!
//! - **Pool Creation**: Initialize new merge pools with primary and replica mints
//! - **Miner Management**: Create and manage merge miners within pools
//! - **Token Operations**: Handle wrapped token minting and distribution
//! - **Type Safety**: Strongly typed account references for all pool components

use anchor_lang::{InstructionData, prelude::*};
use anyhow::Result;
use solana_sdk::instruction::Instruction;
use testsvm::{AccountRef, TestSVM, anchor_instruction};

use crate::{TestMergeMiner, quarry_merge_mine, quarry_mine};

/// Helper for managing a merge pool with type-safe account references
pub struct TestMergePool {
    pub label: String,
    pub pool: AccountRef<quarry_merge_mine::accounts::MergePool>,
    pub primary_mint: AccountRef<anchor_spl::token::Mint>,
    pub replica_mint: AccountRef<anchor_spl::token::Mint>,
}

impl TestMergePool {
    /// Create and set up a new merge pool
    pub fn new(
        env: &mut TestSVM,
        label: &str,
        primary_mint: AccountRef<anchor_spl::token::Mint>,
    ) -> Result<Self> {
        // Calculate merge pool PDA
        let pool = env.get_pda::<quarry_merge_mine::accounts::MergePool>(
            &format!("merge_pool[{label}].pool"),
            &[&"MergePool", &primary_mint.key],
            quarry_merge_mine::ID,
        )?;

        // Calculate replica mint PDA
        let replica_mint = env.get_pda::<anchor_spl::token::Mint>(
            &format!("merge_pool[{label}].replica_mint"),
            &[&"ReplicaMint", &pool.key],
            quarry_merge_mine::ID,
        )?;

        // Create new_pool_v2 instruction
        let instruction = Instruction {
            program_id: quarry_merge_mine::ID,
            accounts: quarry_merge_mine::client::accounts::NewPoolV2 {
                pool: pool.key,
                primary_mint: primary_mint.key,
                replica_mint: replica_mint.key,
                payer: env.default_fee_payer(),
                token_program: anchor_spl::token::ID,
                system_program: solana_sdk::system_program::ID,
                rent: solana_sdk::sysvar::rent::ID,
            }
            .to_account_metas(None),
            data: quarry_merge_mine::client::args::NewPoolV2 {}.data(),
        };

        env.execute_ixs(&[instruction])?;

        Ok(Self {
            label: label.to_string(),
            pool,
            primary_mint,
            replica_mint,
        })
    }

    /// Create a merge miner for this merge pool with the specified owner
    pub fn create_merge_miner(
        &self,
        env: &mut TestSVM,
        label: &str,
        owner: Pubkey,
    ) -> Result<TestMergeMiner> {
        // Calculate merge miner PDA
        let merge_miner = env.get_pda(
            &format!("merge_miner[{label}]"),
            &[&"MergeMiner", &self.pool.key, &owner],
            quarry_merge_mine::ID,
        )?;

        // Create merge miner primary token account ATA
        let (create_mm_primary_ata_ix, primary_tokens) = env.create_ata_ix(
            &format!("merge_miner[{label}].primary_tokens"),
            &merge_miner.into(),
            &self.primary_mint.into(),
        )?;

        // Create merge miner primary token account ATA
        let (create_mm_replica_ata_ix, replica_tokens) = env.create_ata_ix(
            &format!("merge_miner[{label}].replica_tokens"),
            &merge_miner.into(),
            &self.replica_mint.into(),
        )?;

        // Create init_merge_miner_v2 instruction
        let init_merge_miner_ix = anchor_instruction(
            quarry_merge_mine::ID,
            quarry_merge_mine::client::accounts::InitMergeMinerV2 {
                pool: self.pool.key,
                owner,
                mm: merge_miner.key,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_merge_mine::client::args::InitMergeMinerV2 {},
        );

        env.execute_ixs(&[
            create_mm_primary_ata_ix,
            create_mm_replica_ata_ix,
            init_merge_miner_ix,
        ])?;

        Ok(TestMergeMiner {
            merge_miner,
            primary_tokens,
            replica_tokens,
        })
    }

    /// Create necessary token accounts for staking operations
    pub fn setup_staking_accounts(
        &self,
        env: &mut TestSVM,
        merge_miner: &AccountRef<quarry_merge_mine::accounts::MergeMiner>,
        primary_quarry: &Pubkey,
    ) -> Result<()> {
        // Create merge miner primary token account ATA
        let (create_mm_primary_ata_ix, _mm_primary_token_account) = env.create_ata_ix(
            "mm_primary_token_account",
            &merge_miner.into(),
            &self.primary_mint.into(),
        )?;

        // Get primary miner PDA
        let primary_miner = env.get_pda::<quarry_mine::accounts::Miner>(
            "primary_miner",
            &[&"Miner", primary_quarry, &merge_miner],
            crate::quarry_mine::ID,
        )?;

        // Create primary miner vault ATA
        let (create_miner_vault_ix, _primary_miner_vault) = env.create_ata_ix(
            "primary_miner_vault",
            &primary_miner.into(),
            &self.primary_mint.into(),
        )?;

        // Execute the account creation instructions
        env.execute_ixs(&[create_mm_primary_ata_ix, create_miner_vault_ix])?;

        Ok(())
    }
}
