use crate::quarry_mine;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::fmt;
use testsvm::{AccountRef, TestSVM};

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
