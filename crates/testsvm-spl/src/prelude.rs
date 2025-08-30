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

// Re-export testsvm-core prelude
pub use testsvm_core::prelude::*;

// SPL-specific exports  
pub use crate::TestSVMSPLHelpers;