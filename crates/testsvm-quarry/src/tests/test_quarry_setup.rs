/// **TEST: Complete Quarry Integration Test**
///
/// **Purpose:** Test the complete integration flow with the Quarry ecosystem
/// **Code Path:** Full quarry setup using TestRewarder and TestQuarry
/// **Expected Behavior:**
/// - Successfully create all quarry infrastructure components
/// - Verify proper account creation and state
/// - Ensure all programs can interact correctly
/// - Test labeled account tracking
use crate::{TestRewarder, tests::common::init_test_environment};
use anyhow::Result;
use testsvm::prelude::*;

#[test]
fn test_complete_quarry_integration() -> Result<()> {
    // Step 1: Initialize test environment with quarry programs
    let mut env = init_test_environment()?;

    // Step 2: Create authority keypair
    let authority = env.new_wallet("quarry_authority")?;

    // Step 3: Create TestRewarder with labeled accounts
    let test_rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    println!("   ✓ Created rewarder: {}", test_rewarder.rewarder);
    println!("   ✓ Created mint wrapper: {}", test_rewarder.mint_wrapper);
    println!(
        "   ✓ Created reward token: {}",
        test_rewarder.reward_token_mint
    );

    // Step 4: Create staked token mint
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Step 5: Create primary quarry
    println!("\n📍 Creating primary quarry...");
    let primary_quarry = test_rewarder.create_primary_quarry(
        &mut env,
        "primary",
        &staked_token_mint.key,
        &authority,
    )?;

    println!("   ✓ Created primary quarry: {}", primary_quarry.quarry);
    println!("   ✓ Quarry label: {}", primary_quarry.label);

    // Step 6: Create a replica token mint and replica quarry
    let replica_token_mint = env.create_mint("replica_token", 6, &authority.pubkey())?;

    println!("\n📍 Creating replica quarry...");
    let replica_quarry =
        test_rewarder.create_quarry(&mut env, "replica", &replica_token_mint.key, &authority)?;

    println!("   ✓ Created replica quarry: {}", replica_quarry.quarry);
    println!("   ✓ Quarry label: {}", replica_quarry.label);

    // Step 7: Verify accounts that were created
    assert!(
        test_rewarder.reward_token_mint.maybe_load(&env)?.is_some(),
        "Reward token mint should exist"
    );

    assert!(
        test_rewarder.mint_wrapper.maybe_load(&env)?.is_some(),
        "Mint wrapper should exist"
    );

    assert!(
        test_rewarder.rewarder.maybe_load(&env)?.is_some(),
        "Rewarder should exist"
    );

    assert!(
        primary_quarry.quarry.maybe_load(&env)?.is_some(),
        "Primary quarry should exist"
    );

    assert!(
        replica_quarry.quarry.maybe_load(&env)?.is_some(),
        "Replica quarry should exist"
    );

    // Step 8: Verify account data
    println!("\n🔬 Verifying account data...");

    // Fetch and verify mint wrapper
    let mint_wrapper = test_rewarder.fetch_mint_wrapper(&env)?;
    assert_eq!(
        mint_wrapper.token_mint, test_rewarder.reward_token_mint.key,
        "MintWrapper should have correct token mint"
    );
    assert_eq!(
        mint_wrapper.admin,
        authority.pubkey(),
        "MintWrapper should have correct admin"
    );
    println!("   ✓ MintWrapper data verified");

    // Fetch and verify rewarder
    let rewarder = test_rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder.authority,
        authority.pubkey(),
        "Rewarder should have correct authority"
    );
    assert_eq!(
        rewarder.mint_wrapper, test_rewarder.mint_wrapper.key,
        "Rewarder should reference correct mint wrapper"
    );
    println!("   ✓ Rewarder data verified");

    // Fetch and verify primary quarry
    let primary_quarry_data = primary_quarry.fetch_quarry(&env)?;
    assert_eq!(
        primary_quarry_data.token_mint_key, staked_token_mint.key,
        "Primary quarry should have correct token mint"
    );
    assert_eq!(
        primary_quarry_data.rewarder, test_rewarder.rewarder.key,
        "Primary quarry should reference correct rewarder"
    );

    // Fetch and verify replica quarry
    let replica_quarry_data = replica_quarry.fetch_quarry(&env)?;
    assert_eq!(
        replica_quarry_data.token_mint_key, replica_token_mint.key,
        "Replica quarry should have correct token mint"
    );
    assert_eq!(
        replica_quarry_data.rewarder, test_rewarder.rewarder.key,
        "Replica quarry should reference correct rewarder"
    );

    Ok(())
}

#[test]
fn test_merge_miner_integration() -> Result<()> {
    // Step 1: Initialize test environment with quarry programs
    let mut env = init_test_environment()?;

    // Step 2: Create authority keypair
    let authority = env.new_wallet("quarry_authority")?;

    // Step 3: Create TestRewarder with labeled accounts
    let test_rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Step 4: Create staked token mint
    let staked_token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Step 5: Create primary quarry
    println!("\n📍 Creating primary quarry...");
    let primary_quarry = test_rewarder.create_primary_quarry(
        &mut env,
        "primary",
        &staked_token_mint.key,
        &authority,
    )?;

    // Step 6: Create merge pool
    println!("\n🏊 Creating merge pool...");
    let test_merge_pool =
        crate::TestMergePool::new(&mut env, "test_merge_pool", staked_token_mint)?;

    println!("   ✓ Created merge pool: {}", test_merge_pool.pool);
    println!(
        "   ✓ Created replica mint: {}",
        test_merge_pool.replica_mint
    );

    // Step 7: Create merge miner
    let owner = env.new_wallet("merge_miner_owner")?;
    let merge_miner =
        test_merge_pool.create_merge_miner(&mut env, "test_merge_miner", owner.pubkey())?;

    // Step 8: Verify all accounts exist
    assert!(
        test_merge_pool.pool.maybe_load(&env)?.is_some(),
        "Merge pool should exist"
    );

    assert!(
        test_merge_pool.replica_mint.maybe_load(&env)?.is_some(),
        "Replica mint should exist"
    );

    assert!(
        merge_miner.merge_miner.maybe_load(&env)?.is_some(),
        "Merge miner should exist"
    );

    // Step 9: Create replica quarry using TestMergePool
    test_rewarder.create_replica_quarry(&mut env, "replica", &test_merge_pool, &authority)?;

    // Step 10: Verify merge pool data
    let merge_pool_data = test_merge_pool.pool.load(&env)?;
    assert_eq!(
        merge_pool_data.primary_mint, staked_token_mint.key,
        "Merge pool should have correct primary mint"
    );

    // Step 11: Verify merge miner data
    let merge_miner_data = merge_miner.merge_miner.load(&env)?;
    assert_eq!(
        merge_miner_data.pool, test_merge_pool.pool.key,
        "Merge miner should reference correct pool"
    );
    assert_eq!(
        merge_miner_data.owner,
        owner.pubkey(),
        "Merge miner should have correct owner"
    );

    // Step 13: Test staking functionality (create necessary token accounts first)
    // Create ATA for the owner to hold staked tokens
    let (create_owner_ata_ix, owner_token_account) = env.create_ata_ix(
        "owner_staked_tokens",
        &owner.pubkey(),
        &staked_token_mint.into(),
    )?;

    env.execute_ixs(&[create_owner_ata_ix])?;

    // Mint some tokens to the owner
    let mint_ix = anchor_spl::token::spl_token::instruction::mint_to(
        &anchor_spl::token::ID,
        &staked_token_mint.key,
        &owner_token_account.into(),
        &authority.pubkey(),
        &[],
        1_000_000_000, // 1000 tokens with 6 decimals
    )?;

    env.execute_ixs_with_signers(&[mint_ix], &[&authority])?;

    // Test setting up staking accounts
    test_merge_pool.setup_staking_accounts(
        &mut env,
        &merge_miner.merge_miner,
        &primary_quarry.quarry.key,
    )?;

    Ok(())
}
