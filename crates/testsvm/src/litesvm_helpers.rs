use anyhow::Result;
use litesvm::LiteSVM;
use solana_sdk::signature::{Keypair, Signer};

/// Creates a new funded account with the specified amount of lamports
pub fn new_funded_account(svm: &mut LiteSVM, lamports: u64) -> Result<Keypair> {
    let keypair = Keypair::new();

    // Add SOL to the new account
    svm.airdrop(&keypair.pubkey(), lamports).map_err(|e| {
        anyhow::anyhow!(
            "Failed to airdrop {} lamports to account {}: {:?}",
            lamports,
            keypair.pubkey(),
            e
        )
    })?;

    Ok(keypair)
}

