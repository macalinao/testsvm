//! # TestSVM SPL Prelude
//!
//! Common imports for TestSVM SPL users. This module re-exports everything
//! from the testsvm-core prelude plus SPL-specific helpers.
//!
//! # Usage
//!
//! ```rust
//! use testsvm_spl::prelude::*;
//! ```
//!
//! This will import everything from testsvm-core plus:
//! - `TestSVMSPLHelpers` - SPL Token helper trait

pub use crate::TestSVMSPLHelpers;
