use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use testsvm::{AccountRef, TestSVM, anchor_instruction};

use crate::{quarry_merge_mine, quarry_mine};

/// Helper for managing a merge miner with type-safe account references
#[derive(Debug)]
pub struct TestMergeMiner {
    pub merge_miner: AccountRef<quarry_merge_mine::accounts::MergeMiner>,
    pub primary_tokens: AccountRef<anchor_spl::token::TokenAccount>,
    pub replica_tokens: AccountRef<anchor_spl::token::TokenAccount>,
}

impl TestMergeMiner {
    /// Create primary miner and miner vault for the merge miner
    pub fn create_primary_miner(
        &self,
        env: &mut TestSVM,
        label: &str,
        rewarder: &Pubkey,
        quarry: &Pubkey,
        primary_mint: &AccountRef<anchor_spl::token::Mint>,
    ) -> Result<(
        AccountRef<quarry_mine::accounts::Miner>,
        AccountRef<anchor_spl::token::TokenAccount>,
    )> {
        let merge_pool = self.merge_miner.load(env)?.pool;
        // Get primary miner PDA
        let primary_miner = env.get_pda::<quarry_mine::accounts::Miner>(
            &format!("{}_primary_miner", label),
            &[&"Miner", quarry, &self.merge_miner.key],
            crate::quarry_mine::ID,
        )?;

        // Create primary miner vault ATA
        let (create_miner_vault_ix, primary_miner_vault) = env.create_ata_ix(
            &format!("{}_primary_miner_vault", label),
            &primary_miner.into(),
            &primary_mint.into(),
        )?;

        // Create init_miner_v2 instruction for merge mine
        let init_miner_ix = anchor_instruction(
            crate::quarry_merge_mine::ID,
            quarry_merge_mine::client::accounts::InitMinerV2 {
                pool: merge_pool,
                mm: self.merge_miner.key,
                miner: primary_miner.key,
                miner_vault: primary_miner_vault.into(),
                token_mint: primary_mint.key,
                quarry: *quarry,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
                token_program: anchor_spl::token::ID,
                mine_program: crate::quarry_mine::ID,
                rewarder: *rewarder,
            },
            quarry_merge_mine::client::args::InitMinerV2 {},
        );

        // Execute instructions
        env.execute_ixs(&[create_miner_vault_ix, init_miner_ix])?;

        Ok((primary_miner, primary_miner_vault))
    }

    /// Create replica miner and miner vault for the merge miner
    pub fn create_replica_miner(
        &self,
        env: &mut TestSVM,
        label: &str,
        rewarder: &Pubkey,
        quarry: &Pubkey,
        replica_mint: &AccountRef<anchor_spl::token::Mint>,
    ) -> Result<(
        AccountRef<quarry_mine::accounts::Miner>,
        AccountRef<anchor_spl::token::TokenAccount>,
    )> {
        let merge_pool = self.merge_miner.load(env)?.pool;

        // Get replica miner PDA
        let replica_miner = env.get_pda::<quarry_mine::accounts::Miner>(
            &format!("{}_replica_miner", label),
            &[&"Miner", quarry, &self.merge_miner.key],
            crate::quarry_mine::ID,
        )?;

        // Create replica miner vault ATA
        let (create_miner_vault_ix, replica_miner_vault) = env.create_ata_ix(
            &format!("{}_replica_miner_vault", label),
            &replica_miner.into(),
            &replica_mint.into(),
        )?;

        // Create init_miner_v2 instruction for merge mine
        let init_miner_ix = anchor_instruction(
            crate::quarry_merge_mine::ID,
            quarry_merge_mine::client::accounts::InitMinerV2 {
                pool: merge_pool,
                mm: self.merge_miner.key,
                miner: replica_miner.key,
                miner_vault: replica_miner_vault.into(),
                token_mint: replica_mint.key,
                quarry: *quarry,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
                token_program: anchor_spl::token::ID,
                mine_program: crate::quarry_mine::ID,
                rewarder: *rewarder,
            },
            quarry_merge_mine::client::args::InitMinerV2 {},
        );

        // Execute instructions
        env.execute_ixs(&[create_miner_vault_ix, init_miner_ix])?;

        Ok((replica_miner, replica_miner_vault))
    }
}
