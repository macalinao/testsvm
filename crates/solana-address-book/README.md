# Solana Address Book

[![Crates.io](https://img.shields.io/crates/v/solana-address-book.svg)](https://crates.io/crates/solana-address-book)
[![Documentation](https://docs.rs/solana-address-book/badge.svg)](https://docs.rs/solana-address-book)

A Rust library for managing and tracking Solana addresses used in transactions. This crate provides a comprehensive address book system that helps with debugging and transaction analysis by maintaining labeled mappings of Solana public keys to their roles and purposes.

## Features

- **Address Registration**: Register Solana addresses with human-readable labels and specific roles
- **Multiple Role Types**: Support for wallets, mints, ATAs, PDAs, programs, and custom roles
- **Address Lookup**: Quickly find addresses by label, role, or public key
- **Formatted Display**: Colored terminal output for easy address identification
- **Text Replacement**: Replace raw public keys in text with their labeled names
- **Comprehensive Testing**: Full test suite covering all functionality

## Supported Address Types

- **Wallets**: Standard user wallets
- **Mints**: Token mint addresses
- **ATAs**: Associated Token Accounts with mint/owner relationships
- **PDAs**: Program Derived Addresses with seeds and bump information
- **Programs**: Smart contract program addresses
- **Custom**: User-defined roles for specific use cases

## License

Copyright (c) 2025 Ian Macalinao. Licensed under the Apache License, Version 2.0.
