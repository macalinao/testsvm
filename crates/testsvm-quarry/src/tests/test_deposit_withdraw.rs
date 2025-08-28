use anyhow::Result;
use testsvm::prelude::*;

use crate::quarry_mine;
use crate::test_rewarder::TestRewarder;

use super::common::init_test_environment;

#[test]
fn test_deposit_and_withdraw() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and user wallets
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;

    // Create a rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a staked token mint for the quarry
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create quarry for the staked token
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;

    // Set annual rewards rate
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

    // Create miner for the user
    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Initial balance checks
    let initial_user_balance: anchor_spl::token::TokenAccount = user_staked_tokens.load(&env)?;
    let initial_vault_balance: anchor_spl::token::TokenAccount = miner_vault.load(&env)?;

    assert_eq!(
        initial_user_balance.amount,
        1000 * 10u64.pow(6),
        "User should start with 1000 tokens"
    );
    assert_eq!(initial_vault_balance.amount, 0, "Vault should start empty");

    // Deposit/Stake 300 tokens
    let deposit_amount = 300 * 10u64.pow(6);
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        deposit_amount,
        &user,
    )?;

    // Verify balances after deposit
    let user_balance_after_deposit: anchor_spl::token::TokenAccount =
        user_staked_tokens.load(&env)?;
    let vault_balance_after_deposit: anchor_spl::token::TokenAccount = miner_vault.load(&env)?;
    let miner_after_deposit: quarry_mine::accounts::Miner = miner.load(&env)?;

    assert_eq!(
        user_balance_after_deposit.amount,
        700 * 10u64.pow(6),
        "User should have 700 tokens remaining"
    );
    assert_eq!(
        vault_balance_after_deposit.amount,
        300 * 10u64.pow(6),
        "Vault should have 300 tokens"
    );
    assert_eq!(
        miner_after_deposit.balance,
        300 * 10u64.pow(6),
        "Miner should record 300 tokens staked"
    );

    // Withdraw 100 tokens
    let withdraw_amount = 100 * 10u64.pow(6);
    quarry.withdraw_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        withdraw_amount,
        &user,
    )?;

    // Verify balances after withdrawal
    let user_balance_after_withdraw: anchor_spl::token::TokenAccount =
        user_staked_tokens.load(&env)?;
    let vault_balance_after_withdraw: anchor_spl::token::TokenAccount = miner_vault.load(&env)?;
    let miner_after_withdraw: quarry_mine::accounts::Miner = miner.load(&env)?;

    assert_eq!(
        user_balance_after_withdraw.amount,
        800 * 10u64.pow(6),
        "User should have 800 tokens after withdrawal"
    );
    assert_eq!(
        vault_balance_after_withdraw.amount,
        200 * 10u64.pow(6),
        "Vault should have 200 tokens remaining"
    );
    assert_eq!(
        miner_after_withdraw.balance,
        200 * 10u64.pow(6),
        "Miner should record 200 tokens staked"
    );

    println!("✅ Successfully deposited 300 tokens and withdrew 100 tokens");

    Ok(())
}

#[test]
fn test_withdraw_more_than_staked() -> Result<()> {
    let mut env = init_test_environment()?;

    // Setup
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;

    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;

    rewarder.set_annual_rewards_rate(&mut env, 1_000_000 * 10u64.pow(6), &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    let user_staked_tokens = create_and_fund_token_account(
        &mut env,
        "user_staked_tokens",
        &user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Stake 100 tokens
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6),
        &user,
    )?;

    // Try to withdraw 200 tokens (more than staked) - should fail
    let result = quarry.withdraw_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        200 * 10u64.pow(6), // More than staked
        &user,
    );

    assert!(
        result.is_err(),
        "Should fail when trying to withdraw more than staked"
    );

    Ok(())
}

#[test]
fn test_withdraw_wrong_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Setup
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;
    let wrong_user = env.new_wallet("wrong_user")?;

    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;

    rewarder.set_annual_rewards_rate(&mut env, 1_000_000 * 10u64.pow(6), &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    let user_staked_tokens = create_and_fund_token_account(
        &mut env,
        "user_staked_tokens",
        &user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    let wrong_user_tokens = create_and_fund_token_account(
        &mut env,
        "wrong_user_tokens",
        &wrong_user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Stake 100 tokens
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6),
        &user,
    )?;

    // Try to withdraw with wrong authority - should fail
    let result = quarry.withdraw_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &wrong_user_tokens,
        50 * 10u64.pow(6),
        &wrong_user, // Wrong user trying to withdraw
    );

    assert!(
        result.is_err(),
        "Should fail when wrong authority tries to withdraw"
    );

    Ok(())
}

#[test]
fn test_multiple_deposits_and_withdrawals() -> Result<()> {
    let mut env = init_test_environment()?;

    // Setup
    let authority = env.new_wallet("authority")?;
    let user = env.new_wallet("user")?;

    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;
    let quarry = rewarder.create_quarry(&mut env, "main", &staked_token_mint.key, &authority)?;

    rewarder.set_annual_rewards_rate(&mut env, 1_000_000 * 10u64.pow(6), &authority)?;
    let _ = rewarder.new_minter(&mut env, "main", &authority)?;

    let user_staked_tokens = create_and_fund_token_account(
        &mut env,
        "user_staked_tokens",
        &user.pubkey(),
        &staked_token_mint.key,
        1000 * 10u64.pow(6),
        &authority,
    )?;

    let (miner, miner_vault) = quarry.create_miner(&mut env, "user", &user)?;

    // Multiple deposits
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6),
        &user,
    )?;
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        200 * 10u64.pow(6),
        &user,
    )?;
    quarry.stake_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        150 * 10u64.pow(6),
        &user,
    )?;

    // Verify total staked amount
    let miner_after_deposits: quarry_mine::accounts::Miner = miner.load(&env)?;
    assert_eq!(
        miner_after_deposits.balance,
        450 * 10u64.pow(6),
        "Should have 450 tokens staked total"
    );

    // Multiple withdrawals
    quarry.withdraw_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        50 * 10u64.pow(6),
        &user,
    )?;
    quarry.withdraw_tokens(
        &mut env,
        &miner,
        &miner_vault,
        &user_staked_tokens,
        100 * 10u64.pow(6),
        &user,
    )?;

    // Verify final balances
    let miner_final: quarry_mine::accounts::Miner = miner.load(&env)?;
    let user_final: anchor_spl::token::TokenAccount = user_staked_tokens.load(&env)?;
    let vault_final: anchor_spl::token::TokenAccount = miner_vault.load(&env)?;

    assert_eq!(
        miner_final.balance,
        300 * 10u64.pow(6),
        "Should have 300 tokens staked after withdrawals"
    );
    assert_eq!(
        user_final.amount,
        700 * 10u64.pow(6),
        "User should have 700 tokens after all operations"
    );
    assert_eq!(
        vault_final.amount,
        300 * 10u64.pow(6),
        "Vault should have 300 tokens"
    );

    println!("✅ Successfully performed multiple deposits and withdrawals");

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
