use crate::{TestRewarder, quarry_mint_wrapper, tests::common::init_test_environment};
use anyhow::Result;
use testsvm::prelude::*;

#[test]
fn test_create_minter_incorrect_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and unauthorized keypairs
    let authority = env.new_wallet("authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create rewarder with authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a second rewarder that will try to be added as a minter
    let other_rewarder = TestRewarder::new_rewarder(&mut env, "other", &unauthorized)?;

    // Test 1: Unauthorized user cannot create a minter
    let minter_pda: AccountRef<quarry_mint_wrapper::accounts::Minter> = env.get_pda(
        "unauthorized_minter",
        &[
            b"MintWrapperMinter",
            rewarder.mint_wrapper.mint_wrapper.key.as_ref(),
            other_rewarder.rewarder.key.as_ref(),
        ],
        quarry_mint_wrapper::ID,
    )?;

    let create_minter_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::NewMinterV2 {
            new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
                admin: unauthorized.pubkey(), // Wrong authority
            },
            new_minter_authority: other_rewarder.rewarder.key,
            minter: minter_pda.key,
            payer: env.default_fee_payer(),
            system_program: solana_sdk::system_program::ID,
        },
        quarry_mint_wrapper::client::args::NewMinterV2 {},
    );

    // Should fail because unauthorized is not the admin of the mint wrapper
    env.execute_ixs_with_signers(&[create_minter_ix], &[&unauthorized])
        .fails()?
        .with_anchor_error("Unauthorized")?;

    // Test 2: Authority can create a minter
    let new_minter_authority = env.new_wallet("new_minter_authority")?;
    let authorized_minter_pda: AccountRef<quarry_mint_wrapper::accounts::Minter> = env.get_pda(
        "authorized_minter",
        &[
            b"MintWrapperMinter",
            rewarder.mint_wrapper.mint_wrapper.key.as_ref(),
            new_minter_authority.pubkey().as_ref(),
        ],
        quarry_mint_wrapper::ID,
    )?;

    let create_authorized_minter_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::NewMinterV2 {
            new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
                admin: authority.pubkey(), // Correct authority
            },
            new_minter_authority: new_minter_authority.pubkey(),
            minter: authorized_minter_pda.key,
            payer: env.default_fee_payer(),
            system_program: solana_sdk::system_program::ID,
        },
        quarry_mint_wrapper::client::args::NewMinterV2 {},
    );

    env.execute_ixs_with_signers(&[create_authorized_minter_ix], &[&authority])
        .succeeds()?;

    Ok(())
}

#[test]
fn test_update_minter_allowance_incorrect_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and unauthorized keypairs
    let authority = env.new_wallet("authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create a new minter authority for this test
    let new_minter_authority = env.new_wallet("new_minter_authority")?;

    // Create rewarder with authority, but use a custom minter with lower initial allowance
    let mint_wrapper_base = env.new_wallet("mint_wrapper_base")?;
    let _rewarder_base = env.new_wallet("rewarder_base")?;

    // Calculate mint wrapper PDA
    let mint_wrapper: AccountRef<quarry_mint_wrapper::accounts::MintWrapper> = env.get_pda(
        "mint_wrapper",
        &[b"MintWrapper", mint_wrapper_base.pubkey().as_ref()],
        quarry_mint_wrapper::ID,
    )?;

    // Create reward token mint with mint wrapper as authority
    let reward_token_mint = env.create_mint("reward_token", 6, &mint_wrapper.key)?;

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

    // Create minter with manageable initial allowance
    let minter: AccountRef<quarry_mint_wrapper::accounts::Minter> = env.get_pda(
        "minter",
        &[
            b"MintWrapperMinter",
            mint_wrapper.key.as_ref(),
            new_minter_authority.pubkey().as_ref(),
        ],
        quarry_mint_wrapper::ID,
    )?;

    let create_minter_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::NewMinterV2 {
            new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: mint_wrapper.key,
                admin: authority.pubkey(),
            },
            new_minter_authority: new_minter_authority.pubkey(),
            minter: minter.key,
            payer: env.default_fee_payer(),
            system_program: solana_sdk::system_program::ID,
        },
        quarry_mint_wrapper::client::args::NewMinterV2 {},
    );

    let set_initial_allowance_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::MinterUpdate {
            minter: minter.key,
            minter_update_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: mint_wrapper.key,
                admin: authority.pubkey(),
            },
        },
        quarry_mint_wrapper::client::args::MinterUpdate {
            allowance: 1_000_000_000_000, // Set initial allowance to a manageable value
        },
    );

    env.execute_ixs_with_signers(&[create_minter_ix, set_initial_allowance_ix], &[&authority])?;

    // Test 1: Unauthorized user cannot update minter allowance
    let update_allowance_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::MinterUpdate {
            minter: minter.key,
            minter_update_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: mint_wrapper.key,
                admin: unauthorized.pubkey(), // Wrong authority
            },
        },
        quarry_mint_wrapper::client::args::MinterUpdate {
            allowance: 5_000_000_000_000,
        },
    );

    // Should fail because unauthorized is not the admin
    env.execute_ixs_with_signers(&[update_allowance_ix], &[&unauthorized])
        .fails()?;

    // Test 2: Authority can update minter allowance
    let authorized_update_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::MinterUpdate {
            minter: minter.key,
            minter_update_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: mint_wrapper.key,
                admin: authority.pubkey(), // Correct authority
            },
        },
        quarry_mint_wrapper::client::args::MinterUpdate {
            allowance: 10_000_000_000_000,
        },
    );

    env.execute_ixs_with_signers(&[authorized_update_ix], &[&authority])
        .succeeds()?;

    // Verify the allowance was updated
    let minter_data: quarry_mint_wrapper::accounts::Minter = minter.load(&env)?;
    assert_eq!(
        minter_data.allowance, 10_000_000_000_000,
        "Allowance should be updated"
    );

    Ok(())
}

#[test]
fn test_transfer_mint_wrapper_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create original authority and new authority
    let original_authority = env.new_wallet("original_authority")?;
    let new_authority = env.new_wallet("new_authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create rewarder with original authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &original_authority)?;

    // Verify original authority
    let wrapper_data = rewarder.fetch_mint_wrapper(&env)?;
    assert_eq!(
        wrapper_data.admin,
        original_authority.pubkey(),
        "Initial admin should be set"
    );

    // Test 1: Unauthorized user cannot transfer authority
    let unauthorized_transfer_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::TransferAdmin {
            mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
            admin: unauthorized.pubkey(), // Wrong current admin
            next_admin: new_authority.pubkey(),
        },
        quarry_mint_wrapper::client::args::TransferAdmin {},
    );

    env.execute_ixs_with_signers(&[unauthorized_transfer_ix], &[&unauthorized])
        .fails()?
        .with_anchor_error("KeyMismatch")?;

    // Test 2: Original authority can transfer to pending
    let transfer_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::TransferAdmin {
            mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
            admin: original_authority.pubkey(),
            next_admin: new_authority.pubkey(),
        },
        quarry_mint_wrapper::client::args::TransferAdmin {},
    );

    env.execute_ixs_with_signers(&[transfer_ix], &[&original_authority])
        .succeeds()?;

    // Verify pending authority was set
    let wrapper_data = rewarder.fetch_mint_wrapper(&env)?;
    assert_eq!(
        wrapper_data.pending_admin,
        new_authority.pubkey(),
        "Pending admin should be set"
    );
    assert_eq!(
        wrapper_data.admin,
        original_authority.pubkey(),
        "Current admin should remain unchanged"
    );

    // Test 3: Unauthorized user cannot accept authority transfer
    let unauthorized_accept_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::AcceptAdmin {
            mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
            pending_admin: unauthorized.pubkey(), // Wrong pending admin
        },
        quarry_mint_wrapper::client::args::AcceptAdmin {},
    );

    env.execute_ixs_with_signers(&[unauthorized_accept_ix], &[&unauthorized])
        .fails()?
        .with_anchor_error("KeyMismatch")?;

    // Test 4: New authority can accept the transfer
    let accept_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::AcceptAdmin {
            mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
            pending_admin: new_authority.pubkey(),
        },
        quarry_mint_wrapper::client::args::AcceptAdmin {},
    );

    env.execute_ixs_with_signers(&[accept_ix], &[&new_authority])
        .succeeds()?;

    // Verify new authority is active
    let wrapper_data = rewarder.fetch_mint_wrapper(&env)?;
    assert_eq!(
        wrapper_data.admin,
        new_authority.pubkey(),
        "Admin should be transferred"
    );
    assert_eq!(
        wrapper_data.pending_admin,
        Pubkey::default(),
        "Pending admin should be cleared"
    );

    // Test 5: Old authority can no longer make changes (creating a new minter)
    let test_minter_authority = env.new_wallet("test_minter_authority")?;
    let test_minter: AccountRef<quarry_mint_wrapper::accounts::Minter> = env.get_pda(
        "test_minter",
        &[
            b"MintWrapperMinter",
            rewarder.mint_wrapper.mint_wrapper.key.as_ref(),
            test_minter_authority.pubkey().as_ref(),
        ],
        quarry_mint_wrapper::ID,
    )?;

    let old_auth_create_minter_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::NewMinterV2 {
            new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
                admin: original_authority.pubkey(), // Old authority
            },
            new_minter_authority: test_minter_authority.pubkey(),
            minter: test_minter.key,
            payer: env.default_fee_payer(),
            system_program: solana_sdk::system_program::ID,
        },
        quarry_mint_wrapper::client::args::NewMinterV2 {},
    );

    env.execute_ixs_with_signers(&[old_auth_create_minter_ix], &[&original_authority])
        .fails()?
        .with_anchor_error("Unauthorized")?;

    // Test 6: New authority can make changes (create a new minter)
    let new_auth_create_minter_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::NewMinterV2 {
            new_minter_v2_auth: quarry_mint_wrapper::client::accounts::NewMinterAuth {
                mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
                admin: new_authority.pubkey(), // New authority
            },
            new_minter_authority: test_minter_authority.pubkey(),
            minter: test_minter.key,
            payer: env.default_fee_payer(),
            system_program: solana_sdk::system_program::ID,
        },
        quarry_mint_wrapper::client::args::NewMinterV2 {},
    );

    env.execute_ixs_with_signers(&[new_auth_create_minter_ix], &[&new_authority])
        .succeeds()?;

    // Verify the new minter was created
    let test_minter_data: quarry_mint_wrapper::accounts::Minter = test_minter.load(&env)?;
    assert_eq!(
        test_minter_data.minter_authority,
        test_minter_authority.pubkey(),
        "New minter should be created with correct authority"
    );

    Ok(())
}

#[test]
fn test_perform_mint_incorrect_authority() -> Result<()> {
    let mut env = init_test_environment()?;

    // Create authority and unauthorized keypairs
    let authority = env.new_wallet("authority")?;
    let unauthorized = env.new_wallet("unauthorized")?;

    // Create rewarder with authority
    let rewarder = TestRewarder::new_rewarder(&mut env, "main", &authority)?;

    // Create a destination token account
    let destination_owner = env.new_wallet("destination_owner")?;
    let (create_ata_ix, destination) = env.create_ata_ix(
        "destination",
        &destination_owner.pubkey(),
        &rewarder.mint_wrapper.reward_token_mint.into(),
    )?;
    env.execute_ixs(&[create_ata_ix])?;

    // Test: Unauthorized minter authority cannot perform mint
    let mint_amount = 1_000_000_000u64; // 1000 tokens with 6 decimals
    let mint_ix = anchor_instruction(
        quarry_mint_wrapper::ID,
        quarry_mint_wrapper::client::accounts::PerformMint {
            mint_wrapper: rewarder.mint_wrapper.mint_wrapper.key,
            minter_authority: unauthorized.pubkey(), // Wrong minter authority
            token_mint: rewarder.mint_wrapper.reward_token_mint.key,
            destination: destination.into(),
            minter: rewarder.minter.key,
            token_program: anchor_spl::token::ID,
        },
        quarry_mint_wrapper::client::args::PerformMint {
            amount: mint_amount,
        },
    );

    // Should fail because unauthorized is not the minter authority
    env.execute_ixs_with_signers(&[mint_ix], &[&unauthorized])
        .fails()?
        .with_anchor_error("Unauthorized")?;

    // Verify no tokens were minted
    let destination_account: anchor_spl::token::TokenAccount = destination.load(&env)?;
    assert_eq!(
        destination_account.amount, 0,
        "No tokens should be minted for unauthorized mint attempt"
    );

    Ok(())
}

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
