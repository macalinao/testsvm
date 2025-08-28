use crate::{TestRewarder, quarry_mine, tests::common::init_test_environment};
use anyhow::Result;
use solana_sdk::signature::Signer;
use testsvm::TXResultHelpers;

#[test]
fn test_set_rewards_share_authority_check() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and unauthorized keypairs
    let authority = env.new_wallet("authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create rewarder with authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a token mint for the quarry
    let token_mint = env.create_mint("staked_token", 6, &authority.pubkey())?;

    // Create a quarry
    let quarry = rewarder.create_quarry(&mut env, "test_quarry", &token_mint.key, &authority)?;

    // Test 1: Authority can set rewards share
    let new_share = 1000u64;
    let set_share_ix = anchor_instruction(
        quarry_mine::ID,
        quarry_mine::client::accounts::SetRewardsShare {
            auth: quarry_mine::client::accounts::TransferAuthority {
                authority: authority.pubkey(),
                rewarder: rewarder.rewarder.key,
            },
            quarry: quarry.quarry.key,
        },
        quarry_mine::client::args::SetRewardsShare { new_share },
    );

    env.execute_ixs_with_signers(&[set_share_ix.clone()], &[&authority])
        .succeeds()?;

    // Verify the share was updated
    let quarry_data = quarry.fetch_quarry(&env)?;
    assert_eq!(
        quarry_data.rewards_share, new_share,
        "Rewards share should be updated"
    );

    // Test 2: Unauthorized user cannot set rewards share
    let unauthorized_ix = anchor_instruction(
        quarry_mine::ID,
        quarry_mine::client::accounts::SetRewardsShare {
            auth: quarry_mine::client::accounts::TransferAuthority {
                authority: unauthorized.pubkey(),
                rewarder: rewarder.rewarder.key,
            },
            quarry: quarry.quarry.key,
        },
        quarry_mine::client::args::SetRewardsShare { new_share: 2000 },
    );

    env.execute_ixs_with_signers(&[unauthorized_ix], &[&unauthorized])
        .fails()?;

    // Verify share remains unchanged
    let quarry_data = quarry.fetch_quarry(&env)?;
    assert_eq!(
        quarry_data.rewards_share, new_share,
        "Rewards share should remain unchanged after unauthorized attempt"
    );

    Ok(())
}

#[test]
fn test_set_annual_rewards_authority_check() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and unauthorized keypairs
    let authority = env.new_wallet("authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create rewarder with authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Test 1: Authority can set annual rewards
    let annual_rate = 1_000_000_000_000u64; // 1M tokens with 6 decimals
    rewarder
        .set_annual_rewards_rate(&mut env, annual_rate, &authority)
        .succeeds()?;

    // Verify the rate was updated
    let rewarder_data = rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder_data.annual_rewards_rate, annual_rate,
        "Annual rewards rate should be updated"
    );

    // Test 2: Unauthorized user cannot set annual rewards
    rewarder
        .set_annual_rewards_rate(&mut env, 2_000_000_000_000, &unauthorized)
        .fails()?
        .with_anchor_error("Unauthorized")?;

    // Verify rate remains unchanged
    let rewarder_data = rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder_data.annual_rewards_rate, annual_rate,
        "Annual rewards rate should remain unchanged after unauthorized attempt"
    );

    Ok(())
}

#[test]
fn test_transfer_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create original authority and new authority
    let original_authority = env.new_wallet("original_authority")?;
    let new_authority = env.new_wallet("new_authority")?;

    // Create rewarder with original authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &original_authority)?;

    // Verify original authority
    let rewarder_data = rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder_data.authority,
        original_authority.pubkey(),
        "Initial authority should be set"
    );

    // Transfer authority
    let transfer_ix = anchor_instruction(
        quarry_mine::ID,
        quarry_mine::client::accounts::TransferAuthority {
            authority: original_authority.pubkey(),
            rewarder: rewarder.rewarder.key,
        },
        quarry_mine::client::args::TransferAuthority {
            new_authority: new_authority.pubkey(),
        },
    );

    env.execute_ixs_with_signers(&[transfer_ix], &[&original_authority])
        .succeeds()?;

    // Verify authority was transferred
    let rewarder_data = rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder_data.pending_authority,
        new_authority.pubkey(),
        "Pending authority should be set"
    );

    // Accept authority transfer
    let accept_ix = anchor_instruction(
        quarry_mine::ID,
        quarry_mine::client::accounts::AcceptAuthority {
            authority: new_authority.pubkey(),
            rewarder: rewarder.rewarder.key,
        },
        quarry_mine::client::args::AcceptAuthority {},
    );

    env.execute_ixs_with_signers(&[accept_ix], &[&new_authority])
        .succeeds()?;

    // Verify new authority is active
    let rewarder_data = rewarder.fetch_rewarder(&env)?;
    assert_eq!(
        rewarder_data.authority,
        new_authority.pubkey(),
        "Authority should be transferred"
    );
    assert_eq!(
        rewarder_data.pending_authority,
        Pubkey::default(),
        "Pending authority should be cleared"
    );

    // Test that old authority can no longer make changes
    rewarder
        .set_annual_rewards_rate(&mut env, 5_000_000_000_000, &original_authority)
        .fails()?
        .with_anchor_error("Unauthorized")?;

    // Test that new authority can make changes
    rewarder
        .set_annual_rewards_rate(&mut env, 5_000_000_000_000, &new_authority)
        .succeeds()?;

    Ok(())
}

#[test]
fn test_multiple_quarries_management() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority
    let authority = env.new_wallet("authority")?;

    // Create rewarder
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create multiple token mints and quarries
    let num_quarries = 5;
    let mut quarries = Vec::new();

    for i in 0..num_quarries {
        let token_mint = env.create_mint(&format!("token_{}", i), 6, &authority.pubkey())?;
        let quarry = rewarder.create_quarry(
            &mut env,
            &format!("quarry_{}", i),
            &token_mint.key,
            &authority,
        )?;
        quarries.push(quarry);
    }

    // Set different rewards shares for each quarry
    for (i, quarry) in quarries.iter().enumerate() {
        let share = (i as u64 + 1) * 100;

        let set_share_ix = anchor_instruction(
            quarry_mine::ID,
            quarry_mine::client::accounts::SetRewardsShare {
                auth: quarry_mine::client::accounts::TransferAuthority {
                    authority: authority.pubkey(),
                    rewarder: rewarder.rewarder.key,
                },
                quarry: quarry.quarry.key,
            },
            quarry_mine::client::args::SetRewardsShare { new_share: share },
        );

        env.execute_ixs_with_signers(&[set_share_ix], &[&authority])?;

        // Verify the share was set correctly
        let quarry_data = quarry.fetch_quarry(&env)?;
        assert_eq!(
            quarry_data.rewards_share, share,
            "Quarry {} should have share {}",
            i, share
        );
    }

    // Verify all quarries maintain their individual shares
    for (i, quarry) in quarries.iter().enumerate() {
        let quarry_data = quarry.fetch_quarry(&env)?;
        let expected_share = (i as u64 + 1) * 100;
        assert_eq!(
            quarry_data.rewards_share, expected_share,
            "Quarry {} share should remain {}",
            i, expected_share
        );
    }

    Ok(())
}

use solana_sdk::pubkey::Pubkey;

fn anchor_instruction<T: anchor_lang::InstructionData + anchor_lang::Discriminator>(
    program_id: Pubkey,
    accounts: impl anchor_lang::ToAccountMetas,
    data: T,
) -> solana_sdk::instruction::Instruction {
    solana_sdk::instruction::Instruction::new_with_bytes(
        program_id,
        &data.data(),
        accounts.to_account_metas(None),
    )
}
