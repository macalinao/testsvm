//! # Rewarder Testing Utilities
//!
//! Test helpers for Quarry rewarder management.
//!
//! This module provides the `TestRewarder` struct for managing reward distribution
//! systems in the Quarry protocol. Rewarders control the minting and distribution
//! of reward tokens to miners based on their staked amounts and the configured
//! reward rates.
//!
//! ## Features
//!
//! - **Rewarder Creation**: Initialize new rewarders with mint wrappers
//! - **Quarry Management**: Create and configure quarries under the rewarder
//! - **Reward Configuration**: Set annual reward rates and distribution parameters
//! - **Authority Control**: Manage rewarder authority and pause states

use crate::{quarry_mine, quarry_mint_wrapper};
use anchor_lang::prelude::*;
use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use testsvm::{AccountRef, TXResult, TestSVM, anchor_instruction};

/// Test rewarder with labeled accounts
pub struct TestRewarder {
    pub label: String,
    pub rewarder: AccountRef<quarry_mine::accounts::Rewarder>,
    pub mint_wrapper: AccountRef<quarry_mint_wrapper::accounts::MintWrapper>,
    pub minter: AccountRef<quarry_mint_wrapper::accounts::Minter>,
    pub reward_token_mint: AccountRef<anchor_spl::token::Mint>,
    pub claim_fee_token_account: AccountRef<anchor_spl::token::TokenAccount>,
    pub authority: Pubkey,

    // Keep the base keypairs for signing
    pub mint_wrapper_base: Keypair,
    pub rewarder_base: Keypair,
}

impl TestRewarder {
    /// Create a new rewarder with the specified label and authority
    pub fn new_rewarder(env: &mut TestSVM, label: &str, authority: &Keypair) -> Result<Self> {
        let mint_wrapper_base =
            env.new_wallet(&format!("rewarder[{label}].mint_wrapper_base"))?;
        let rewarder_base = env.new_wallet(&format!("rewarder[{label}].rewarder_base"))?;

        // Calculate mint wrapper PDA
        let (mint_wrapper, mint_wrapper_bump) = env.find_pda_with_bump(
            &format!("rewarder[{label}].mint_wrapper"),
            &[&"MintWrapper", &mint_wrapper_base.pubkey()],
            quarry_mint_wrapper::ID,
        )?;

        // Create reward token mint with mint wrapper as authority
        let reward_token_mint = env
            .create_mint(
                &format!("rewarder[{label}].reward_token"),
                6,
                &mint_wrapper,
            )
            .context("Failed to create reward token mint")?;

        let create_wrapper_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewWrapper {
                base: mint_wrapper_base.pubkey(),
                mint_wrapper,
                admin: authority.pubkey(),
                token_mint: reward_token_mint.key,
                token_program: anchor_spl::token::ID,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mint_wrapper::client::args::NewWrapper {
                bump: mint_wrapper_bump,
                hard_cap: u64::MAX,
            },
        );

        env.execute_ixs_with_signers(&[create_wrapper_ix], &[&mint_wrapper_base])?;

        // Calculate rewarder PDA
        let rewarder = env.get_pda(
            &format!("rewarder[{label}].rewarder"),
            &[&"Rewarder", &rewarder_base.pubkey()],
            quarry_mine::ID,
        )?;

        // Create claim fee token account for the rewarder
        let (create_ata_ix, claim_fee_token_account) = env.create_ata_ix(
            &format!("rewarder[{label}].claim_fee_tokens"),
            &rewarder.key,
            &reward_token_mint.key,
        )?;

        env.execute_ixs(&[create_ata_ix])?;

        // Create rewarder
        let create_rewarder_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::NewRewarderV2 {
                base: rewarder_base.pubkey(),
                rewarder: rewarder.key,
                initial_authority: authority.pubkey(),
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
                mint_wrapper,
                rewards_token_mint: reward_token_mint.key,
                claim_fee_token_account: claim_fee_token_account.key,
            },
            quarry_mine::client::args::NewRewarderV2 {},
        );

        env.execute_ixs_with_signers(&[create_rewarder_ix], &[&rewarder_base])?;

        // Create and approve minter for the mint wrapper
        let minter = env.get_pda(
            &format!("rewarder[{label}].minter"),
            &[&"MintWrapperMinter", &mint_wrapper, &rewarder],
            quarry_mint_wrapper::ID,
        )?;

        let create_minter_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewMinterV2 {
                new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper,
                    admin: authority.pubkey(),
                },
                new_minter_authority: rewarder.key,
                minter: minter.key,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mint_wrapper::client::args::NewMinterV2 {},
        );

        let set_allowance_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::MinterUpdate {
                minter: minter.key,
                minter_update_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper,
                    admin: authority.pubkey(),
                },
            },
            quarry_mint_wrapper::client::args::MinterUpdate {
                allowance: u64::MAX,
            },
        );

        env.execute_ixs_with_signers(&[create_minter_ix, set_allowance_ix], &[authority])?;

        Ok(TestRewarder {
            label: label.to_string(),
            rewarder,
            mint_wrapper: AccountRef::new(mint_wrapper),
            minter,
            reward_token_mint,
            authority: authority.pubkey(),
            mint_wrapper_base,
            rewarder_base,
            claim_fee_token_account,
        })
    }

    /// Create a replica quarry for this rewarder
    pub fn create_quarry(
        &self,
        env: &mut TestSVM,
        quarry_name: &str,
        staked_token_mint: &Pubkey,
        authority: &Keypair,
    ) -> Result<crate::TestQuarry> {
        let quarry_label = format!("rewarder[{}].quarry[{}]", self.label, quarry_name);

        // Calculate quarry PDA
        let quarry = env.get_pda_key(
            &format!("{quarry_label}.quarry"),
            &[&"Quarry", &self.rewarder.key, staked_token_mint],
            quarry_mine::ID,
        )?;

        // Create quarry
        use quarry_mine::client::accounts::TransferAuthority;

        let create_quarry_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::CreateQuarryV2 {
                quarry,
                auth: TransferAuthority {
                    authority: authority.pubkey(),
                    rewarder: self.rewarder.key,
                },
                token_mint: *staked_token_mint,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mine::client::args::CreateQuarryV2 {},
        );

        let set_shares_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::SetRewardsShare {
                auth: TransferAuthority {
                    authority: authority.pubkey(),
                    rewarder: self.rewarder.key,
                },
                quarry,
            },
            quarry_mine::client::args::SetRewardsShare { new_share: 1 },
        );

        env.execute_ixs_with_signers(&[create_quarry_ix, set_shares_ix], &[authority])?;

        Ok(crate::TestQuarry {
            label: quarry_label,
            quarry: AccountRef::new(quarry),
            rewarder: self.rewarder.key,
            staked_token_mint: AccountRef::new(*staked_token_mint),
        })
    }

    /// Create a primary quarry for this rewarder
    pub fn create_primary_quarry(
        &self,
        env: &mut TestSVM,
        quarry_name: &str,
        staked_token_mint: &Pubkey,
        authority: &Keypair,
    ) -> Result<crate::TestQuarry> {
        // Primary quarries have the same logic as replica quarries
        // The distinction is mainly semantic/organizational
        self.create_quarry(env, quarry_name, staked_token_mint, authority)
    }

    /// Create a replica quarry using the replica mint from a TestMergePool
    pub fn create_replica_quarry(
        &self,
        env: &mut TestSVM,
        quarry_name: &str,
        merge_pool: &crate::TestMergePool,
        authority: &Keypair,
    ) -> Result<crate::TestQuarry> {
        // Use the replica mint from the merge pool
        self.create_quarry(env, quarry_name, &merge_pool.replica_mint.key, authority)
    }

    /// Fetch the Rewarder account from chain
    pub fn fetch_rewarder(&self, env: &TestSVM) -> Result<quarry_mine::accounts::Rewarder> {
        self.rewarder.load(env)
    }

    /// Fetch the MintWrapper account from chain
    pub fn fetch_mint_wrapper(
        &self,
        env: &TestSVM,
    ) -> Result<quarry_mint_wrapper::accounts::MintWrapper> {
        self.mint_wrapper.load(env)
    }

    /// Helper to set annual rewards rate for a quarry
    pub fn set_annual_rewards_rate(
        &self,
        env: &mut TestSVM,
        annual_rate: u64,
        authority: &Keypair,
    ) -> TXResult {
        let set_rewards_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::SetAnnualRewards {
                auth: quarry_mine::client::accounts::TransferAuthority {
                    authority: authority.pubkey(),
                    rewarder: self.rewarder.key,
                },
            },
            quarry_mine::client::args::SetAnnualRewards {
                new_rate: annual_rate,
            },
        );
        env.execute_ixs_with_signers(&[set_rewards_ix], &[authority])
    }

    /// Create a new minter to allow minting
    pub fn new_minter(
        &self,
        env: &mut TestSVM,
        label: &str,
        authority: &Keypair,
    ) -> Result<TXResult> {
        // Calculate the minter PDA but don't add to address book yet
        let (minter, minter_bump) = Pubkey::find_program_address(
            &[
                b"MintWrapperMinter",
                self.mint_wrapper.key.as_ref(),
                self.rewarder.key.as_ref(),
            ],
            &quarry_mint_wrapper::ID,
        );

        let new_minter_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewMinter {
                new_minter_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper: self.mint_wrapper.key,
                    admin: authority.pubkey(),
                },
                new_minter_authority: self.rewarder.key,
                minter,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mint_wrapper::client::args::NewMinter { bump: minter_bump },
        );

        let result = env.execute_ixs_with_signers(&[new_minter_ix], &[authority]);

        // Add the minter to address book after creation
        if result.is_ok() {
            env.address_book.add_pda(
                minter,
                format!("rewarder[{}].minter[{}]", self.label, label),
                vec![
                    "MintWrapperMinter".to_string(),
                    self.mint_wrapper.key.to_string(),
                    self.rewarder.key.to_string(),
                ],
                quarry_mint_wrapper::ID,
                minter_bump,
            )?;
        }

        Ok(result)
    }
}
