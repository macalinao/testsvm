//! # TestSVM Prelude
//!
//! Common imports for TestSVM users.
//!
//! This module re-exports the most commonly used types and traits from the TestSVM
//! framework, allowing users to import everything they need with a single use statement:
//!
//! ```rust
//! use testsvm::prelude::*;
//! ```

// Core TestSVM types
pub use crate::{
    AccountRef, TXError, TXErrorAssertions, TXResult, TXResultAssertions, TXSuccessAssertions,
    TestSVM,
};

pub use crate::TestSVMSPLHelpers;
pub use anchor_spl;
pub use testsvm_core::prelude::*;
