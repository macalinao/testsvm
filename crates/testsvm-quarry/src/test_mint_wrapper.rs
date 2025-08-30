//! # Test Mint Wrapper
//!
//! Testing utilities for the Quarry Mint Wrapper program.
//!
//! This module provides the `TestMintWrapper` struct which simplifies the creation
//! and management of mint wrappers in test environments. It handles:
//!
//! - **Mint Wrapper Creation**: Initialize mint wrappers with proper configuration
//! - **Token Management**: Create and manage reward token mints
//! - **Minter Management**: Create and configure minters with allowances
//! - **Authority Control**: Manage mint wrapper authority and permissions

use crate::quarry_mint_wrapper;
use anchor_lang::prelude::*;
use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use testsvm::prelude::*;

/// Test mint wrapper with labeled accounts
pub struct TestMintWrapper {
    pub label: String,
    pub mint_wrapper: AccountRef<quarry_mint_wrapper::accounts::MintWrapper>,
    pub mint_wrapper_base: Keypair,
    pub reward_token_mint: AccountRef<anchor_spl::token::Mint>,
    pub authority: Pubkey,
}

impl TestMintWrapper {
    /// Create a new mint wrapper with the specified label and authority
    pub fn new(env: &mut TestSVM, label: &str, authority: &Keypair) -> Result<Self> {
        let mint_wrapper_base = env.new_wallet(&format!("mint_wrapper[{label}].base"))?;

        // Calculate mint wrapper PDA
        let mint_wrapper: AccountRef<quarry_mint_wrapper::accounts::MintWrapper> = env.get_pda(
            &format!("mint_wrapper[{label}]"),
            &[b"MintWrapper", mint_wrapper_base.pubkey().as_ref()],
            quarry_mint_wrapper::ID,
        )?;

        // Create reward token mint with mint wrapper as authority
        let reward_token_mint = env
            .create_mint(
                &format!("mint_wrapper[{label}].reward_token"),
                6,
                &mint_wrapper.key,
            )
            .context("Failed to create reward token mint")?;

        // Create the mint wrapper
        let create_wrapper_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewWrapperV2 {
                base: mint_wrapper_base.pubkey(),
                mint_wrapper: mint_wrapper.key,
                admin: authority.pubkey(),
                token_mint: reward_token_mint.key,
                token_program: anchor_spl::token::ID,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mint_wrapper::client::args::NewWrapperV2 { hard_cap: u64::MAX },
        );

        env.execute_ixs_with_signers(&[create_wrapper_ix], &[&mint_wrapper_base])?;

        Ok(TestMintWrapper {
            label: label.to_string(),
            mint_wrapper,
            mint_wrapper_base,
            reward_token_mint,
            authority: authority.pubkey(),
        })
    }

    /// Create a minter for the specified authority with the given allowance
    pub fn create_minter(
        &self,
        env: &mut TestSVM,
        minter_authority: &Pubkey,
        allowance: u64,
        admin: &Keypair,
    ) -> Result<AccountRef<quarry_mint_wrapper::accounts::Minter>> {
        // Calculate minter PDA
        let minter = env.get_pda(
            &format!("mint_wrapper[{}].minter[{}]", self.label, minter_authority),
            &[
                b"MintWrapperMinter",
                self.mint_wrapper.key.as_ref(),
                minter_authority.as_ref(),
            ],
            quarry_mint_wrapper::ID,
        )?;

        // Create the minter
        let create_minter_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::NewMinterV2 {
                new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper: self.mint_wrapper.key,
                    admin: admin.pubkey(),
                },
                new_minter_authority: *minter_authority,
                minter: minter.key,
                payer: env.default_fee_payer(),
                system_program: solana_sdk::system_program::ID,
            },
            quarry_mint_wrapper::client::args::NewMinterV2 {},
        );

        // Set the allowance
        let set_allowance_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::MinterUpdate {
                minter: minter.key,
                minter_update_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                    mint_wrapper: self.mint_wrapper.key,
                    admin: admin.pubkey(),
                },
            },
            quarry_mint_wrapper::client::args::MinterUpdate { allowance },
        );

        env.execute_ixs_with_signers(&[create_minter_ix, set_allowance_ix], &[admin])?;

        Ok(minter)
    }

    /// Transfer mint wrapper authority to a new authority
    pub fn transfer_authority(
        &mut self,
        env: &mut TestSVM,
        new_authority: &Pubkey,
        current_authority: &Keypair,
    ) -> Result<()> {
        let transfer_authority_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::TransferAdmin {
                mint_wrapper: self.mint_wrapper.key,
                admin: current_authority.pubkey(),
                next_admin: *new_authority,
            },
            quarry_mint_wrapper::client::args::TransferAdmin {},
        );

        env.execute_ixs_with_signers(&[transfer_authority_ix], &[current_authority])?;

        // Update the authority field
        self.authority = *new_authority;

        Ok(())
    }

    /// Accept mint wrapper authority transfer
    pub fn accept_authority(&mut self, env: &mut TestSVM, new_authority: &Keypair) -> Result<()> {
        let accept_authority_ix = anchor_instruction(
            quarry_mint_wrapper::ID,
            quarry_mint_wrapper::client::accounts::AcceptAdmin {
                mint_wrapper: self.mint_wrapper.key,
                pending_admin: new_authority.pubkey(),
            },
            quarry_mint_wrapper::client::args::AcceptAdmin {},
        );

        env.execute_ixs_with_signers(&[accept_authority_ix], &[new_authority])?;

        // Update the authority field
        self.authority = new_authority.pubkey();

        Ok(())
    }
}
