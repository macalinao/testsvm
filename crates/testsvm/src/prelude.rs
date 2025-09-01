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
pub use anchor_spl;
pub use testsvm_assertions::{TXErrorAssertions, TXResultAssertions, TXSuccessAssertions};
pub use testsvm_core::prelude::*;
pub use testsvm_spl::prelude::*;
