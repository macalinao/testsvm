# anchor-utils

[![Crates.io](https://img.shields.io/crates/v/anchor-utils.svg)](https://crates.io/crates/anchor-utils)
[![Documentation](https://docs.rs/anchor-utils/badge.svg)](https://docs.rs/anchor-utils)

Utility functions for working with Anchor programs in Solana.

## Features

- `anchor_instruction` - Helper function to create Solana instructions from Anchor's generated `declare_program!` client structs

## Usage

```rust
use anchor_utils::anchor_instruction;
use anchor_lang::{InstructionData, ToAccountMetas};

// Create an instruction using Anchor's generated types
let instruction = anchor_instruction(
    program_id,
    accounts_struct, // implements ToAccountMetas
    instruction_data // implements InstructionData
);
```

## License

Apache-2.0