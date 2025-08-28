# testsvm-quarry

[![Crates.io](https://img.shields.io/crates/v/testsvm-quarry.svg)](https://crates.io/crates/testsvm-quarry)
[![Documentation](https://docs.rs/testsvm-quarry/badge.svg)](https://docs.rs/testsvm-quarry)

Testing utilities for the Quarry protocol on Solana using the TestSVM framework.

## Overview

This crate provides testing utilities and helpers for interacting with the Quarry mining protocol in a test environment. It includes functions for setting up Quarry programs, managing mining operations, and testing reward distribution mechanisms.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
testsvm-quarry = "0.1.0"
```

## Setup

Before using this crate, you need to download the Quarry program binaries. Run the following commands to fetch the required programs:

```bash
# Set your project root directory
export ROOT_DIR=/path/to/your/project

# Download Quarry programs
solana program dump QMMD16kjauP5knBwxNUJRZ1Z5o3deBuFrqVjBVmmqto $ROOT_DIR/fixtures/programs/quarry_merge_mine.so
solana program dump QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB $ROOT_DIR/fixtures/programs/quarry_mine.so
solana program dump QMWoBmAyJLAsA1Lh9ugMTw2gciTihncciphzdNzdZYV $ROOT_DIR/fixtures/programs/quarry_mint_wrapper.so
```

## Usage

```rust
use testsvm::TestSVM;
use testsvm_quarry::setup::setup_quarry_programs;

fn main() -> anyhow::Result<()> {
    let mut env = TestSVM::new();
    
    // Setup Quarry programs
    setup_quarry_programs(&mut env)?;
    
    // Your test logic here
    
    Ok(())
}
```

## Features

- **Program Setup**: Easy setup of Quarry mining programs in test environment
- **Mining Operations**: Test mining rewards and distributions
- **Merge Mining**: Support for testing merge mining functionality
- **Mint Wrapper**: Testing utilities for the Quarry mint wrapper

## Program IDs

The crate includes the following Quarry program IDs:

- **Quarry Mine**: `QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB`
- **Quarry Merge Mine**: `QMMD16kjauP5knBwxNUJRZ1Z5o3deBuFrqVjBVmmqto`
- **Quarry Mint Wrapper**: `QMWoBmAyJLAsA1Lh9ugMTw2gciTihncciphzdNzdZYV`

## Dependencies

This crate depends on:
- `testsvm` - Core testing framework for Solana SVM
- `anchor-lang` - Anchor framework for Solana program development
- `solana-sdk` - Solana SDK for blockchain interactions

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

## Author

Ian Macalinao <me@ianm.com>

## Repository

[https://github.com/macalinao/testsvm](https://github.com/macalinao/testsvm)