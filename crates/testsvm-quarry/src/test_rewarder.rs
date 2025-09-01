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

use crate::{TestMintWrapper, quarry_mine, quarry_mint_wrapper};
use testsvm::prelude::*;

/// Test rewarder with labeled accounts
pub struct TestRewarder {
    pub label: String,
    pub rewarder: AccountRef<quarry_mine::accounts::Rewarder>,
    pub mint_wrapper: TestMintWrapper,
    pub minter: AccountRef<quarry_mint_wrapper::accounts::Minter>,
    pub claim_fee_token_account: AccountRef<anchor_spl::token::TokenAccount>,
    pub authority: Pubkey,

    // Keep the base keypair for signing
    pub rewarder_base: Keypair,
}

impl TestRewarder {
    /// Create a new rewarder with the specified label and authority
    pub fn new_rewarder(env: &mut TestSVM, label: &str, authority: &Keypair) -> Result<Self> {
        // Create the mint wrapper using TestMintWrapper
        let mint_wrapper = TestMintWrapper::new(env, &format!("rewarder[{label}]"), authority)?;

        let rewarder_base = env.new_wallet(&format!("rewarder[{label}].rewarder_base"))?;

        // Calculate rewarder PDA
        let rewarder = env.get_pda(
            &format!("rewarder[{label}].rewarder"),
            &[b"Rewarder", rewarder_base.pubkey().as_ref()],
            quarry_mine::ID,
        )?;

        // Create claim fee token account for the rewarder
        let (create_ata_ix, claim_fee_token_account) = env.create_ata_ix(
            &format!("rewarder[{label}].claim_fee_tokens"),
            &rewarder.key,
            &mint_wrapper.reward_token_mint.key,
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
                mint_wrapper: mint_wrapper.mint_wrapper.key,
                rewards_token_mint: mint_wrapper.reward_token_mint.key,
                claim_fee_token_account: claim_fee_token_account.key,
            },
            quarry_mine::client::args::NewRewarderV2 {},
        );

        env.execute_ixs_with_signers(&[create_rewarder_ix], &[&rewarder_base])?;

        // Create minter for the rewarder with max allowance
        let minter = mint_wrapper.create_minter(env, &rewarder.key, u64::MAX, authority)?;

        Ok(TestRewarder {
            label: label.to_string(),
            rewarder,
            mint_wrapper,
            minter,
            authority: authority.pubkey(),
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
        let quarry = env.get_pda(
            &format!("{quarry_label}.quarry"),
            &[
                b"Quarry",
                self.rewarder.key.as_ref(),
                staked_token_mint.as_ref(),
            ],
            quarry_mine::ID,
        )?;

        // Create quarry
        use quarry_mine::client::accounts::TransferAuthority;

        let create_quarry_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::CreateQuarryV2 {
                quarry: quarry.key,
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
                quarry: quarry.key,
            },
            quarry_mine::client::args::SetRewardsShare { new_share: 1 },
        );

        env.execute_ixs_with_signers(&[create_quarry_ix, set_shares_ix], &[authority])?;

        Ok(crate::TestQuarry {
            label: quarry_label,
            quarry,
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
        self.mint_wrapper.mint_wrapper.load(env)
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
                self.mint_wrapper.mint_wrapper.key.as_ref(),
                self.rewarder.key.as_ref(),
            ],
            &quarry_mint_wrapper::ID,
        );

        let new_minter_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewMinter {
                new_minter_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper: self.mint_wrapper.mint_wrapper.key,
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
                    self.mint_wrapper.mint_wrapper.key.to_string(),
                    self.rewarder.key.to_string(),
                ],
                quarry_mint_wrapper::ID,
                minter_bump,
            )?;
        }

        Ok(result)
    }
}
