# testsvm-assertions

[![Crates.io](https://img.shields.io/crates/v/testsvm-assertions.svg)](https://crates.io/crates/testsvm-assertions)
[![Documentation](https://docs.rs/testsvm-assertions/badge.svg)](https://docs.rs/testsvm-assertions)

Assertion helpers for testing transaction results in TestSVM.

This crate provides traits and types for asserting expected transaction outcomes, including methods for verifying that transactions succeed or fail with specific errors. These assertions are particularly useful in test environments where you need to verify that your program behaves correctly under various conditions.

## Features

- **Success/Failure Assertions**: Verify transactions succeed or fail as expected
- **Error Matching**: Check for specific error types including Anchor errors
- **Type-safe API**: Compile-time guarantees for assertion chains

## License

Copyright (c) 2025 Ian Macalinao. Licensed under the Apache License, Version 2.0.
