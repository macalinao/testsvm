<h1 align="center">
TestSVM
</h1>

<p align="center">
Robust testing framework for Solana programs, written in Rust.
</p>

TestSVM is a testing framework for Solana programs, written in Rust. Built on top of [LiteSVM](https://github.com/LiteSVM/litesvm), it is an order of magnitude faster than traditional `solana-test-validator` + JavaScript tests.

Features include:

- Account labeling and logging, so you can easily track what `Pubkey` maps to what account.
- Assertions for transaction results: assert that a transaction fails with a specific error, or that it succeeds.
- Nicely formatted transaction logs with account labeling, so you know what accounts have issues in a transaction.
- Type-safe utilities for validating state and loading it from the SVM.

**Note: this is a work in progress and is subject to change.**

## Examples

The `testsvm-quarry` crate contains a number of tests that demonstrate all TestSVM features in [tests/](crates/testsvm-quarry/src/tests/).

I plan to add more examples in the future.

## Crates

There are two main crates that compose TestSVM:

- `testsvm` - Core testing framework
- `testsvm-quarry` - Helpers for testing programs that integrate with Quarry

### Internal crates

The following crates are used internally by TestSVM, but are not documented or intended for public use.

- `address-book` - Keeps track of all the addresses used in the tests
- `anchor-utils` - Lightweight utilities for interacting with programs that use Anchor.

## License

Copyright (c) 2025 Ian Macalinao. Licensed under the Apache License, Version 2.0.
