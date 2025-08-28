<h1 align="center">
TestSVM
</h1>

[<img alt="github" src="https://img.shields.io/badge/github-macalinao/testsvm-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/macalinao/testsvm)
[<img alt="crates.io" src="https://img.shields.io/crates/v/testsvm.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/testsvm)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-testsvm-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/testsvm/latest/testsvm/)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/macalinao/testsvm/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/macalinao/testsvm/actions?query=branch%3Amaster)

### TestSVM is a blazing fast testing framework for Solana programs, written in Rust.

TestSVM provides:

- Account labeling and logging, so you can easily track what `Pubkey` maps to what account.
- Assertions for transaction results: assert that a transaction fails with a specific error, or that it succeeds.
- Nicely formatted transaction logs with account labeling, so you know what accounts have issues in a transaction.
- Type-safe utilities for validating state and loading it from the SVM.

Built on top of [LiteSVM](https://github.com/LiteSVM/litesvm), tests written using TestSVM are an order of magnitude faster than traditional `solana-test-validator` + JavaScript tests.

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

- `solana-address-book` - Keeps track of all the addresses used in the tests
- `anchor-utils` - Lightweight utilities for interacting with programs that use Anchor.

## License

Copyright (c) 2025 Ian Macalinao. Licensed under the Apache License, Version 2.0.
