# Address Book

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

## Usage

```rust
use address_book::{AddressBook, RegisteredAddress, AddressRole};
use solana_sdk::pubkey::Pubkey;

// Create a new address book
let mut book = AddressBook::new();

// Add different types of addresses
let wallet_key = Pubkey::new_unique();
book.add_wallet(wallet_key, "user_wallet".to_string()).unwrap();

let mint_key = Pubkey::new_unique();
book.add_mint(mint_key, "my_token".to_string()).unwrap();

let program_key = Pubkey::new_unique();
book.add_program(program_key, "my_program").unwrap();

// Add default Solana programs
book.add_default_accounts().unwrap();

// Look up addresses
let label = book.get_label(&wallet_key);
println!("Wallet label: {}", label);

// Format addresses with colors for terminal display
let formatted = book.format_address(&wallet_key);
println!("Formatted: {}", formatted);

// Find addresses by role
let all_wallets = book.get_all_by_role_type("wallet");
println!("Found {} wallet addresses", all_wallets.len());

// Print the entire address book
book.print_all();
```

## Use Cases

This library is particularly useful for:

- **Transaction Debugging**: Replace cryptic public keys with meaningful labels in transaction logs
- **Development Tools**: Build CLI tools that need to track and display Solana addresses
- **Testing Frameworks**: Maintain address mappings during integration tests
- **Block Explorers**: Provide human-readable address information
- **Wallet Applications**: Display user-friendly names for known addresses

## Dependencies

- `solana-sdk`: Core Solana types and utilities
- `anchor-lang`: Anchor framework integration
- `colored`: Terminal color output
- `anyhow`: Error handling
- `strum`: Enum string serialization

## License

MIT