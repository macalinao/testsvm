# TestSVM SPL

[![Crates.io](https://img.shields.io/crates/v/testsvm-spl.svg)](https://crates.io/crates/testsvm-spl)
[![Documentation](https://docs.rs/testsvm-spl/badge.svg)](https://docs.rs/testsvm-spl)

SPL Token helper functions for the TestSVM testing framework. This crate provides the `TestSVMSPLHelpers` trait that extends TestSVM with comprehensive SPL Token functionality for creating mints, token accounts, and performing token operations in test environments.

## Features

- **Mint Creation**: Create SPL token mints with automatic address book registration
- **ATA Management**: Create and manage Associated Token Accounts with proper labeling
- **Token Operations**: Helper functions for common token operations like minting and transfers
- **Address Book Integration**: Automatic registration of mints and ATAs for debugging
- **Authority Management**: Support for mint and freeze authority configuration
- **Decimals Support**: Full support for tokens with configurable decimal places

## Core Components

- **TestSVMSPLHelpers**: Trait extending TestSVM with SPL Token functionality
- **Mint Creation**: Create mints with automatic rent calculation and initialization
- **ATA Instructions**: Generate instructions for Associated Token Account creation
- **Address Registration**: Automatic labeling of token-related accounts

## License

Copyright (c) 2025 Ian Macalinao. Licensed under the Apache License, Version 2.0.