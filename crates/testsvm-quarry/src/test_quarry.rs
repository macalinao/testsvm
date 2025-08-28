use crate::{TestRewarder, quarry_mine, quarry_mint_wrapper};
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::fmt;
use testsvm::{AccountRef, TestSVM, anchor_instruction};

/// Test quarry with labeled accounts
#[derive(Debug)]
pub struct TestQuarry {
    pub label: String,
    pub quarry: AccountRef<quarry_mine::accounts::Quarry>,
    pub rewarder: Pubkey,
    pub staked_token_mint: AccountRef<anchor_spl::token::Mint>,
}

impl TestQuarry {
    /// Fetch the Quarry account from chain
    pub fn fetch_quarry(&self, env: &TestSVM) -> Result<quarry_mine::accounts::Quarry> {
        self.quarry.load(env)
    }

    /// Create a miner for a user
    pub fn create_miner(
        &self,
        env: &mut TestSVM,
        label: &str,
        user: &Keypair,
    ) -> Result<(
        AccountRef<quarry_mine::accounts::Miner>,
        AccountRef<anchor_spl::token::TokenAccount>,
    )> {
        let miner = env.get_pda(
            &format!("miner_{}", label),
            &[b"Miner", &self.quarry.key.as_ref(), &user.pubkey().as_ref()],
            quarry_mine::ID,
        )?;

        // Create the miner vault ATA first
        let (create_vault_ix, miner_vault) = env.create_ata_ix(
            &format!("miner_vault_{}", label),
            &miner.key,
            &self.staked_token_mint.key,
        )?;
        env.execute_ixs(&[create_vault_ix])?;

        let create_miner_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::CreateMinerV2 {
                quarry: self.quarry.key,
                rewarder: self.rewarder,
                miner: miner.key,
                authority: user.pubkey(),
                payer: env.default_fee_payer(),
                token_mint: self.staked_token_mint.key,
                miner_vault: miner_vault.key,
                system_program: solana_sdk::system_program::ID,
                token_program: anchor_spl::token::ID,
            },
            quarry_mine::client::args::CreateMinerV2 {},
        );

        env.execute_ixs_with_signers(&[create_miner_ix], &[user])?;

        Ok((miner, miner_vault))
    }

    /// Stake tokens into the miner
    pub fn stake_tokens(
        &self,
        env: &mut TestSVM,
        miner: &AccountRef<quarry_mine::accounts::Miner>,
        miner_vault: &AccountRef<anchor_spl::token::TokenAccount>,
        user_token_account: &AccountRef<anchor_spl::token::TokenAccount>,
        amount: u64,
        user: &Keypair,
    ) -> Result<()> {
        let stake_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::StakeTokens {
                authority: user.pubkey(),
                miner: miner.key,
                quarry: self.quarry.key,
                rewarder: self.rewarder,
                token_account: user_token_account.key,
                miner_vault: miner_vault.key,
                token_program: anchor_spl::token::ID,
            },
            quarry_mine::client::args::StakeTokens { amount },
        );

        env.execute_ixs_with_signers(&[stake_ix], &[user])?;

        Ok(())
    }

    /// Update quarry rewards to reflect time passage
    pub fn update_quarry_rewards(&self, env: &mut TestSVM) -> Result<()> {
        let update_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::UpdateQuarryRewards {
                quarry: self.quarry.key,
                rewarder: self.rewarder,
            },
            quarry_mine::client::args::UpdateQuarryRewards {},
        );

        env.execute_ixs(&[update_ix])?;

        Ok(())
    }

    /// Withdraw tokens from the miner
    pub fn withdraw_tokens(
        &self,
        env: &mut TestSVM,
        miner: &AccountRef<quarry_mine::accounts::Miner>,
        miner_vault: &AccountRef<anchor_spl::token::TokenAccount>,
        user_token_account: &AccountRef<anchor_spl::token::TokenAccount>,
        amount: u64,
        user: &Keypair,
    ) -> Result<()> {
        let withdraw_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::WithdrawTokens {
                authority: user.pubkey(),
                miner: miner.key,
                quarry: self.quarry.key,
                rewarder: self.rewarder,
                token_account: user_token_account.key,
                miner_vault: miner_vault.key,
                token_program: anchor_spl::token::ID,
            },
            quarry_mine::client::args::WithdrawTokens { amount },
        );

        env.execute_ixs_with_signers(&[withdraw_ix], &[user])?;

        Ok(())
    }

    /// Claim rewards for a miner
    pub fn claim_rewards(
        &self,
        env: &mut TestSVM,
        rewarder: &TestRewarder,
        miner: &AccountRef<quarry_mine::accounts::Miner>,
        _miner_vault: &AccountRef<anchor_spl::token::TokenAccount>,
        user_rewards_account: &AccountRef<anchor_spl::token::TokenAccount>,
        user: &Keypair,
    ) -> Result<()> {
        // Find the minter PDA - it should already exist from perform_new_minter
        let (minter, _) = Pubkey::find_program_address(
            &[
                b"MintWrapperMinter",
                rewarder.mint_wrapper.key.as_ref(),
                rewarder.rewarder.key.as_ref(),
            ],
            &quarry_mint_wrapper::ID,
        );

        let claim_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::ClaimRewardsV2 {
                mint_wrapper: rewarder.mint_wrapper.key,
                mint_wrapper_program: quarry_mint_wrapper::ID,
                minter,
                rewards_token_mint: rewarder.reward_token_mint.key,
                rewards_token_account: user_rewards_account.key,
                claim_fee_token_account: rewarder.claim_fee_token_account.key,
                claim: quarry_mine::client::accounts::Claim {
                    authority: user.pubkey(),
                    miner: miner.key,
                    quarry: self.quarry.key,
                    token_program: anchor_spl::token::ID,
                    rewarder: self.rewarder,
                },
            },
            quarry_mine::client::args::ClaimRewardsV2 {},
        );

        env.execute_ixs_with_signers(&[claim_ix], &[user])?;

        Ok(())
    }
}

impl fmt::Display for TestQuarry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.quarry.key)
    }
}

impl AsRef<[u8]> for TestQuarry {
    fn as_ref(&self) -> &[u8] {
        self.quarry.key.as_ref()
    }
}
