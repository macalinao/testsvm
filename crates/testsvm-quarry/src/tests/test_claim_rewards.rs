use anyhow::Result;
use testsvm::prelude::*;

use crate::quarry_mine;
use crate::test_rewarder::TestRewarder;

use super::common::init_test_environment;

#[test]
fn test_claim_rewards() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and user wallets
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;

    // Create a rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a staked token mint for the quarry
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create quarry for the staked token using V2 instruction
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;

    // Set annual rewards (1,000,000 tokens per year with 6 decimals)
    let annual_rewards_rate = 1_000_000 * 10u64.pow(6);
    rewarder.set_annual_rewards_rate(&mut env, annual_rewards_rate, &authority)?;

    // Create new minter to allow minting
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    // Create user's staked token account and mint tokens
    let user_staked_tokens = create_and_fund_token_account(
        &mut env,
        "user_staked_tokens",
        &user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6), // 1000 tokens
        &authority,
    )?;

    // Create miner for the user (this also creates the miner vault)
    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Stake tokens
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6), // Stake 100 tokens
        &user,
    )?;

    // Verify initial miner state
    let miner_account: quarry_mine::accounts::Miner = miner.load(&env)?;
    assert_eq!(
        miner_account.balance,
        100 * 10u64.pow(6),
        "Miner should have 100 staked tokens"
    );
    assert_eq!(
        miner_account.rewards_earned, 0,
        "No rewards should be earned initially"
    );

    // Advance time by 1 year to accumulate rewards
    env.advance_time(365 * 24 * 60 * 60); // 1 year in seconds

    // Update quarry rewards (this updates the accumulated rewards per token)
    quarry.update_quarry_rewards(&mut env)?;

    // Create user's reward token account
    let (create_ata_ix, user_rewards_account) = env.create_ata_ix(
        "user_rewards",
        &user.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.key,
    )?;
    env.execute_ixs(&[create_ata_ix])?;

    // Claim rewards
    quarry.claim_rewards(
        &mut env,
        &rewarder,
        &miner,
        &miner_vault,
        &user_rewards_account,
        &user,
    )?;

    // Verify rewards were claimed
    let reward_account: anchor_spl::token::TokenAccount = user_rewards_account.load(&env)?;
    assert!(
        reward_account.amount > 0,
        "User should have received rewards"
    );

    // Calculate expected rewards (approximately)
    // User staked 100 tokens out of 100 total (100% of pool)
    // Annual rewards: 1,000,000 tokens
    // Expected: ~1,000,000 tokens (minus any rounding)
    let expected_min_rewards = 999_000 * 10u64.pow(6); // Allow for some rounding
    assert!(
        reward_account.amount >= expected_min_rewards,
        "User should receive approximately 1M tokens. Got: {}",
        reward_account.amount
    );

    println!(
        "✅ Successfully claimed {} reward tokens",
        reward_account.amount / 10u64.pow(6)
    );

    // Verify miner rewards were reset after claiming
    let miner_after_claim: quarry_mine::accounts::Miner = miner.load(&env)?;
    assert_eq!(
        miner_after_claim.rewards_earned, 0,
        "Miner rewards should be reset after claiming"
    );

    Ok(())
}

#[test]
fn test_claim_rewards_wrong_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and users
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;
    let wrong_user = env.new_wallet("wrong_user")?;

    // Create a rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a staked token mint for the quarry
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create quarry and setup
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;
    rewarder.set_annual_rewards_rate(&mut env, 1_000_000 * 10u64.pow(6), &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    // Create miner for the user
    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Fund user's token account and stake
    let user_staked_tokens = create_and_fund_token_account(
        &mut env,
        "user_staked_tokens",
        &user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6),
        &user,
    )?;

    // Advance time and update rewards
    env.advance_time(365 * 24 * 60 * 60);
    quarry.update_quarry_rewards(&mut env)?;

    // Create reward token account for wrong user
    let (create_ata_ix, wrong_user_rewards) = env.create_ata_ix(
        "wrong_user_rewards",
        &wrong_user.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.key,
    )?;
    env.execute_ixs(&[create_ata_ix])?;

    // Try to claim rewards with wrong authority - should fail
    let result = quarry.claim_rewards(
        &mut env,
        &rewarder,
        &miner,
        &miner_vault,
        &wrong_user_rewards,
        &wrong_user, // Wrong user trying to claim
    );

    assert!(
        result.is_err(),
        "Should fail when wrong authority tries to claim"
    );

    Ok(())
}

#[test]
fn test_claim_rewards_no_stake() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and user
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;

    // Create a rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a staked token mint for the quarry
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create quarry and setup
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;
    rewarder.set_annual_rewards_rate(&mut env, 1_000_000 * 10u64.pow(6), &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    // Create miner but don't stake anything
    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Advance time
    env.advance_time(365 * 24 * 60 * 60);
    quarry.update_quarry_rewards(&mut env)?;

    // Create reward token account
    let (create_ata_ix, user_rewards) = env.create_ata_ix(
        "user_rewards",
        &user.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.key,
    )?;
    env.execute_ixs(&[create_ata_ix])?;

    // Claim rewards - should succeed but get 0 rewards
    quarry.claim_rewards(
        &mut env,
        &rewarder,
        &miner,
        &miner_vault,
        &user_rewards,
        &user,
    )?;

    // Verify no rewards were received
    let reward_account: anchor_spl::token::TokenAccount = user_rewards.load(&env)?;
    assert_eq!(
        reward_account.amount, 0,
        "User should receive 0 rewards when no stake"
    );

    Ok(())
}

#[test]
fn test_claim_rewards_multiple_users() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and multiple users
    let authority = env.new_wallet("authority")?;
    let user1 = env.new_wallet("user1")?;
    let user2 = env.new_wallet("user2")?;

    // Create a rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a staked token mint for the quarry
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create quarry and setup
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;
    let annual_rewards = 1_000_000 * 10u64.pow(6);
    rewarder.set_annual_rewards_rate(&mut env, annual_rewards, &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    // Create miners for both users
    let (miner1, vault1) = quarry.create_miner(&mut env, "user1", &user1)?;
    let (miner2, vault2) = quarry.create_miner(&mut env, "user2", &user2)?;

    // Fund both users' token accounts
    let user1_tokens = create_and_fund_token_account(
        &mut env,
        "user1_tokens",
        &user1.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    let user2_tokens = create_and_fund_token_account(
        &mut env,
        "user2_tokens",
        &user2.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    // User1 stakes 300 tokens, User2 stakes 200 tokens
    quarry.stake_tokens(
        &mut env,
        &miner1,
        &vault1,
        &user1_tokens,
        300 * 10u64.pow(6),
        &user1,
    )?;
    quarry.stake_tokens(
        &mut env,
        &miner2,
        &vault2,
        &user2_tokens,
        200 * 10u64.pow(6),
        &user2,
    )?;

    // Advance time by 1 year
    env.advance_time(365 * 24 * 60 * 60);
    quarry.update_quarry_rewards(&mut env)?;

    // Create reward accounts
    let (ix1, rewards1) = env.create_ata_ix(
        "rewards1",
        &user1.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.key,
    )?;
    let (ix2, rewards2) = env.create_ata_ix(
        "rewards2",
        &user2.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.key,
    )?;
    env.execute_ixs(&[ix1, ix2])?;

    // Both users claim rewards
    quarry.claim_rewards(&mut env, &rewarder, &miner1, &vault1, &rewards1, &user1)?;
    quarry.claim_rewards(&mut env, &rewarder, &miner2, &vault2, &rewards2, &user2)?;

    // Verify proportional rewards
    let rewards1_account: anchor_spl::token::TokenAccount = rewards1.load(&env)?;
    let rewards2_account: anchor_spl::token::TokenAccount = rewards2.load(&env)?;

    // User1 staked 60% (300/500), User2 staked 40% (200/500)
    // Allow for rounding errors
    let user1_expected = (annual_rewards * 60 / 100) * 999 / 1000; // 0.1% tolerance
    let user2_expected = (annual_rewards * 40 / 100) * 999 / 1000; // 0.1% tolerance

    assert!(
        rewards1_account.amount >= user1_expected,
        "User1 should receive ~60% of rewards. Got: {}, Expected min: {}",
        rewards1_account.amount,
        user1_expected
    );

    assert!(
        rewards2_account.amount >= user2_expected,
        "User2 should receive ~40% of rewards. Got: {}, Expected min: {}",
        rewards2_account.amount,
        user2_expected
    );

    println!(
        "✅ User1 claimed {} tokens (60% stake)",
        rewards1_account.amount / 10u64.pow(6)
    );
    println!(
        "✅ User2 claimed {} tokens (40% stake)",
        rewards2_account.amount / 10u64.pow(6)
    );

    Ok(())
}

fn create_and_fund_token_account(
    env: &mut TestSVM,
    label: &str,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    mint_authority: &Keypair,
) -> Result<AccountRef<anchor_spl::token::TokenAccount>> {
    // Create ATA
    let (create_ata_ix, token_account) = env.create_ata_ix(label, owner, mint)?;
    env.execute_ixs(&[create_ata_ix])?;

    // Mint tokens
    let mint_to_ix = anchor_spl::token::spl_token::instruction::mint_to(
        &anchor_spl::token::ID,
        mint,
        &token_account.key,
        &mint_authority.pubkey(),
        &[],
        amount,
    )?;

    env.execute_ixs_with_signers(&[mint_to_ix], &[mint_authority])?;

    Ok(token_account)
}
