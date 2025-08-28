//! Registered address types and utilities for the address book.

use crate::pda_seeds::{SeedPart, find_pda_with_bump_and_strings};
use anchor_lang::prelude::*;

/// Role type for registered addresses, defining the purpose of each address
#[derive(Debug, Clone, strum::Display, Hash, PartialEq, Eq)]
pub enum AddressRole {
    /// Standard user wallet
    #[strum(serialize = "wallet")]
    Wallet,

    /// Token mint address
    #[strum(serialize = "mint")]
    Mint,

    /// Associated Token Account with mint and owner references
    #[strum(serialize = "ata")]
    Ata { mint: Pubkey, owner: Pubkey },

    /// Program Derived Address with seeds and program information
    #[strum(serialize = "pda")]
    Pda {
        seeds: Vec<String>,
        program_id: Pubkey,
        bump: u8,
    },

    /// Smart contract program address
    #[strum(serialize = "program")]
    Program,

    /// Custom user-defined role
    #[strum(serialize = "custom")]
    Custom(String),
}

/// Registered address with role information
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RegisteredAddress {
    /// The address.
    pub key: Pubkey,
    /// The address's function within the program.
    pub role: AddressRole,
}

impl RegisteredAddress {
    /// Creates a new registered address with the specified role
    ///
    /// # Arguments
    /// * `address` - The public key to register
    /// * `role` - The role that defines the address's purpose
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::{RegisteredAddress, AddressRole};
    ///
    /// let address = Pubkey::new_unique();
    /// let registered = RegisteredAddress::new(address, AddressRole::Wallet);
    /// ```
    pub fn new(address: Pubkey, role: AddressRole) -> Self {
        Self { key: address, role }
    }

    /// Creates a wallet-type registered address
    ///
    /// # Arguments
    /// * `address` - The wallet's public key
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let wallet = Pubkey::new_unique();
    /// let registered = RegisteredAddress::wallet(wallet);
    /// ```
    pub fn wallet(address: Pubkey) -> Self {
        Self::new(address, AddressRole::Wallet)
    }

    /// Creates a mint-type registered address
    ///
    /// # Arguments
    /// * `address` - The mint's public key
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let mint = Pubkey::new_unique();
    /// let registered = RegisteredAddress::mint(mint);
    /// ```
    pub fn mint(address: Pubkey) -> Self {
        Self::new(address, AddressRole::Mint)
    }

    /// Creates an Associated Token Account (ATA) registered address
    ///
    /// # Arguments
    /// * `address` - The ATA's public key
    /// * `mint` - The token mint public key
    /// * `owner` - The owner's public key
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let ata = Pubkey::new_unique();
    /// let mint = Pubkey::new_unique();
    /// let owner = Pubkey::new_unique();
    /// let registered = RegisteredAddress::ata(ata, mint, owner);
    /// ```
    pub fn ata(address: Pubkey, mint: Pubkey, owner: Pubkey) -> Self {
        Self::new(address, AddressRole::Ata { mint, owner })
    }

    /// Creates a custom-role registered address
    ///
    /// # Arguments
    /// * `address` - The address's public key
    /// * `custom_role` - A custom string describing the address's role
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let address = Pubkey::new_unique();
    /// let registered = RegisteredAddress::custom(address, "governance");
    /// ```
    pub fn custom(address: Pubkey, custom_role: &str) -> Self {
        Self::new(address, AddressRole::Custom(custom_role.to_string()))
    }

    /// Creates a program-type registered address
    ///
    /// # Arguments
    /// * `address` - The program's public key
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let program = Pubkey::new_unique();
    /// let registered = RegisteredAddress::program(program);
    /// ```
    pub fn program(address: Pubkey) -> Self {
        Self::new(address, AddressRole::Program)
    }

    /// Creates a PDA registered address by finding the program address
    ///
    /// # Arguments
    /// * `seeds` - The seeds used to derive the PDA
    /// * `program_id` - The program that owns the PDA
    ///
    /// # Returns
    /// A tuple containing:
    /// * The derived PDA public key
    /// * The bump seed
    /// * The registered address
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::{RegisteredAddress, SeedPart};
    ///
    /// let program_id = Pubkey::new_unique();
    /// let user = Pubkey::new_unique();
    /// let seeds: Vec<&dyn SeedPart> = vec![&"vault", &user];
    ///
    /// let (pda_key, bump, registered) = RegisteredAddress::pda(&seeds, &program_id);
    /// ```
    pub fn pda(seeds: &[&dyn SeedPart], program_id: &Pubkey) -> (Pubkey, u8, Self) {
        let derived_pda = find_pda_with_bump_and_strings(seeds, program_id);

        (
            derived_pda.key,
            derived_pda.bump,
            Self::new(
                derived_pda.key,
                AddressRole::Pda {
                    seeds: derived_pda.seed_strings,
                    program_id: *program_id,
                    bump: derived_pda.bump,
                },
            ),
        )
    }

    /// Creates a PDA registered address from existing PDA information
    ///
    /// # Arguments
    /// * `pubkey` - The PDA's public key
    /// * `seeds` - The string representations of seeds used
    /// * `program_id` - The program that owns the PDA
    /// * `bump` - The bump seed
    ///
    /// # Example
    /// ```
    /// use anchor_lang::prelude::*;
    /// use address_book::RegisteredAddress;
    ///
    /// let pda = Pubkey::new_unique();
    /// let program_id = Pubkey::new_unique();
    /// let seeds = vec!["vault".to_string(), "v1".to_string()];
    ///
    /// let registered = RegisteredAddress::pda_from_parts(pda, seeds, program_id, 255);
    /// ```
    pub fn pda_from_parts(
        pubkey: Pubkey,
        seeds: Vec<String>,
        program_id: Pubkey,
        bump: u8,
    ) -> Self {
        Self::new(
            pubkey,
            AddressRole::Pda {
                seeds,
                program_id,
                bump,
            },
        )
    }
}

impl std::fmt::Display for RegisteredAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.role {
            AddressRole::Ata { mint, owner } => {
                write!(f, "{} [ata mint:{} owner:{}]", self.key, mint, owner)
            }
            AddressRole::Pda { seeds, bump, .. } => {
                write!(
                    f,
                    "{} [pda seeds:{} bump:{}]",
                    self.key,
                    seeds.join(","),
                    bump
                )
            }
            AddressRole::Custom(custom) => {
                write!(f, "{} [{}]", self.key, custom)
            }
            _ => {
                write!(f, "{} [{}]", self.key, self.role)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let pubkey = Pubkey::new_unique();
        let registered = RegisteredAddress::wallet(pubkey);

        assert_eq!(registered.key, pubkey);
        assert!(matches!(registered.role, AddressRole::Wallet));
    }

    #[test]
    fn test_mint_creation() {
        let pubkey = Pubkey::new_unique();
        let registered = RegisteredAddress::mint(pubkey);

        assert_eq!(registered.key, pubkey);
        assert!(matches!(registered.role, AddressRole::Mint));
    }

    #[test]
    fn test_ata_creation() {
        let ata = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let registered = RegisteredAddress::ata(ata, mint, owner);

        assert_eq!(registered.key, ata);
        if let AddressRole::Ata { mint: m, owner: o } = registered.role {
            assert_eq!(m, mint);
            assert_eq!(o, owner);
        } else {
            panic!("Expected ATA role");
        }
    }

    #[test]
    fn test_program_creation() {
        let pubkey = Pubkey::new_unique();
        let registered = RegisteredAddress::program(pubkey);

        assert_eq!(registered.key, pubkey);
        assert!(matches!(registered.role, AddressRole::Program));
    }

    #[test]
    fn test_custom_creation() {
        let pubkey = Pubkey::new_unique();
        let registered = RegisteredAddress::custom(pubkey, "governance");

        assert_eq!(registered.key, pubkey);
        if let AddressRole::Custom(role) = registered.role {
            assert_eq!(role, "governance");
        } else {
            panic!("Expected Custom role");
        }
    }

    #[test]
    fn test_pda_creation() {
        let program_id = Pubkey::new_unique();
        let seeds: Vec<&dyn SeedPart> = vec![&"test", &"seed"];

        let (pubkey, bump, registered) = RegisteredAddress::pda(&seeds, &program_id);

        assert_eq!(registered.key, pubkey);
        if let AddressRole::Pda {
            seeds: pda_seeds,
            program_id: pda_program_id,
            bump: pda_bump,
        } = &registered.role
        {
            assert_eq!(pda_seeds, &vec!["test".to_string(), "seed".to_string()]);
            assert_eq!(*pda_program_id, program_id);
            assert_eq!(*pda_bump, bump);
        } else {
            panic!("Expected PDA role");
        }
    }
}
