use anchor_lang::prelude::*;

pub mod setup;
pub mod test_merge_miner;
pub mod test_merge_pool;
pub mod test_quarry;
pub mod test_rewarder;

pub use setup::*;
pub use test_merge_miner::*;
pub use test_merge_pool::*;
pub use test_quarry::*;
pub use test_rewarder::*;

// Declare quarry programs using their IDLs
declare_program!(quarry_merge_mine);
declare_program!(quarry_mine);
declare_program!(quarry_mint_wrapper);

#[cfg(test)]
mod tests;
