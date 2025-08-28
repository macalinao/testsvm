pub mod pda_seeds;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anyhow::{Result, anyhow};
use colored::*;
use std::collections::{HashMap, HashSet};

pub use pda_seeds::{
    DerivedPda, SeedPart, find_pda_with_bump, find_pda_with_bump_and_strings, seed_to_string,
};

/// Role type for registered addresses
#[derive(Debug, Clone, strum::Display, Hash, PartialEq, Eq)]
pub enum AddressRole {
    #[strum(serialize = "wallet")]
    Wallet,
    #[strum(serialize = "mint")]
    Mint,
    #[strum(serialize = "ata")]
    Ata { mint: Pubkey, owner: Pubkey },
    #[strum(serialize = "pda")]
    Pda {
        seeds: Vec<String>,
        program_id: Pubkey,
        bump: u8,
    },
    #[strum(serialize = "program")]
    Program,
    #[strum(serialize = "custom")]
    Custom(String),
}

/// Registered address with label and role information
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RegisteredAddress {
    pub address: Pubkey,
    pub label: String,
    pub role: AddressRole,
}

impl RegisteredAddress {
    pub fn new(address: Pubkey, label: String, role: AddressRole) -> Self {
        Self {
            address,
            label,
            role,
        }
    }

    pub fn wallet(address: Pubkey, label: &str) -> Self {
        Self::new(address, label.to_string(), AddressRole::Wallet)
    }

    pub fn mint(address: Pubkey, label: &str) -> Self {
        Self::new(address, label.to_string(), AddressRole::Mint)
    }

    pub fn ata(address: Pubkey, label: &str, mint: Pubkey, owner: Pubkey) -> Self {
        Self::new(address, label.to_string(), AddressRole::Ata { mint, owner })
    }

    pub fn custom(address: Pubkey, label: &str, custom_role: &str) -> Self {
        Self::new(
            address,
            label.to_string(),
            AddressRole::Custom(custom_role.to_string()),
        )
    }

    pub fn program(address: Pubkey, label: &str) -> Self {
        Self::new(address, label.to_string(), AddressRole::Program)
    }

    pub fn pda<T>(label: &str, seeds: &[T], program_id: &Pubkey) -> (Pubkey, u8, Self)
    where
        T: AsRef<[u8]> + ToString,
    {
        let (pubkey, bump) = Pubkey::find_program_address(
            seeds
                .iter()
                .map(|seed| seed.as_ref())
                .collect::<Vec<_>>()
                .as_slice(),
            program_id,
        );
        (
            pubkey,
            bump,
            Self::new(
                pubkey,
                label.to_string(),
                AddressRole::Pda {
                    seeds: seeds.iter().map(|seed| seed.to_string()).collect(),
                    program_id: *program_id,
                    bump,
                },
            ),
        )
    }
}

impl std::fmt::Display for RegisteredAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.role {
            AddressRole::Ata { mint, owner } => {
                write!(f, "{} [ata mint:{} owner:{}]", self.label, mint, owner)
            }
            AddressRole::Pda { seeds, bump, .. } => {
                write!(
                    f,
                    "{} [pda seeds:{} bump:{}]",
                    self.label,
                    seeds.join(","),
                    bump
                )
            }
            AddressRole::Custom(custom) => {
                write!(f, "{} [{}]", self.label, custom)
            }
            _ => {
                write!(f, "{} [{}]", self.label, self.role)
            }
        }
    }
}

/// Address book for mapping public keys to registered addresses
/// This helps with debugging by providing context for addresses in transactions
#[derive(Debug, Default)]
pub struct AddressBook {
    addresses: HashMap<Pubkey, Vec<RegisteredAddress>>,
    registered_addresses: HashSet<RegisteredAddress>,
    labels: HashMap<String, RegisteredAddress>,
}

impl AddressBook {
    /// Create a new empty address book
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
            registered_addresses: HashSet::new(),
            labels: HashMap::new(),
        }
    }

    pub fn add_default_accounts(&mut self) -> Result<()> {
        self.add_program(system_program::ID, "system_program")?;
        self.add_program(anchor_spl::token::ID, "token_program")?;
        self.add_program(anchor_spl::associated_token::ID, "associated_token_program")?;
        Ok(())
    }

    pub fn get_label(&self, pubkey: &Pubkey) -> String {
        self.addresses
            .get(pubkey)
            .and_then(|v| v.first())
            .map(|r| r.label.clone())
            .unwrap_or_else(|| pubkey.to_string())
    }

    /// Add an address with a registered address to the address book
    /// Returns an error if the label already exists
    pub fn add(&mut self, pubkey: Pubkey, registered_address: RegisteredAddress) -> Result<()> {
        // Check if this label already exists
        if let Some(existing_address) = self.labels.get(&registered_address.label) {
            if existing_address.address != pubkey
                || existing_address.role != registered_address.role
            {
                return Err(anyhow!(
                    "Label '{}' already exists in address book",
                    registered_address.label
                ));
            }
            return Ok(());
        }

        // Add to labels and registered addresses
        self.labels
            .insert(registered_address.label.clone(), registered_address.clone());
        self.registered_addresses.insert(registered_address.clone());

        // Add to addresses vector (allows multiple registrations per pubkey)
        self.addresses
            .entry(pubkey)
            .or_default()
            .push(registered_address);

        Ok(())
    }

    /// Add an address with a simple label (defaults to wallet role)
    pub fn add_wallet(&mut self, pubkey: Pubkey, label: String) -> Result<()> {
        self.add(pubkey, RegisteredAddress::wallet(pubkey, &label))
    }

    /// Add a mint address
    pub fn add_mint(&mut self, pubkey: Pubkey, label: String) -> Result<()> {
        self.add(pubkey, RegisteredAddress::mint(pubkey, &label))
    }

    /// Add an ATA address
    pub fn add_ata(
        &mut self,
        pubkey: Pubkey,
        label: String,
        mint: Pubkey,
        owner: Pubkey,
    ) -> Result<()> {
        self.add(pubkey, RegisteredAddress::ata(pubkey, &label, mint, owner))
    }

    /// Add a custom role address
    pub fn add_custom(&mut self, pubkey: Pubkey, label: String, custom_role: String) -> Result<()> {
        self.add(
            pubkey,
            RegisteredAddress::custom(pubkey, &label, &custom_role),
        )
    }

    /// Add a PDA address
    pub fn add_pda(
        &mut self,
        pubkey: Pubkey,
        label: String,
        seeds: Vec<String>,
        program_id: Pubkey,
        bump: u8,
    ) -> Result<()> {
        self.add(
            pubkey,
            RegisteredAddress::new(
                pubkey,
                label,
                AddressRole::Pda {
                    seeds,
                    program_id,
                    bump,
                },
            ),
        )
    }

    /// Add a program address
    pub fn add_program(&mut self, pubkey: Pubkey, label: &str) -> Result<()> {
        self.add(pubkey, RegisteredAddress::program(pubkey, label))
    }

    /// Find a PDA with bump and add it to the address book
    pub fn find_pda_with_bump(
        &mut self,
        label: &str,
        seeds: &[&dyn SeedPart],
        program_id: Pubkey,
    ) -> Result<(Pubkey, u8)> {
        // Use the helper function from pda_seeds module
        let derived_pda = find_pda_with_bump_and_strings(seeds, &program_id);

        // Add to address book
        self.add_pda(
            derived_pda.key,
            label.to_string(),
            derived_pda.seed_strings,
            program_id,
            derived_pda.bump,
        )?;

        Ok((derived_pda.key, derived_pda.bump))
    }

    /// Get the registered addresses for a pubkey, if they exist
    pub fn get(&self, pubkey: &Pubkey) -> Option<&Vec<RegisteredAddress>> {
        self.addresses.get(pubkey)
    }

    /// Get the first registered address for a pubkey, if it exists
    pub fn get_first(&self, pubkey: &Pubkey) -> Option<&RegisteredAddress> {
        self.addresses.get(pubkey).and_then(|v| v.first())
    }

    /// Get an address by its role
    pub fn get_by_role(&self, role: &AddressRole) -> Option<Pubkey> {
        for registered in self.registered_addresses.iter() {
            if &registered.role == role {
                return Some(registered.address);
            }
        }
        None
    }

    /// Get all addresses with a specific role type (e.g., all wallets)
    pub fn get_all_by_role_type(&self, role_type: &str) -> Vec<Pubkey> {
        let mut addresses = Vec::new();
        for registered in self.registered_addresses.iter() {
            match (&registered.role, role_type) {
                (AddressRole::Wallet, "wallet")
                | (AddressRole::Mint, "mint")
                | (AddressRole::Program, "program") => {
                    addresses.push(registered.address);
                }
                (AddressRole::Ata { .. }, "ata") => {
                    addresses.push(registered.address);
                }
                (AddressRole::Pda { .. }, "pda") => {
                    addresses.push(registered.address);
                }
                (AddressRole::Custom(_), "custom") => {
                    addresses.push(registered.address);
                }
                _ => {}
            }
        }
        addresses
    }

    /// Get a formatted string representation of an address with colors
    /// If the address is in the book, returns a colored formatted string
    /// Otherwise, just returns the address as a string
    pub fn format_address(&self, pubkey: &Pubkey) -> String {
        match self.get_first(pubkey) {
            Some(registered_address) => match &registered_address.role {
                AddressRole::Wallet => format!(
                    "{} {}",
                    registered_address.label.bright_cyan().bold(),
                    "[wallet]".to_string().dimmed()
                ),
                AddressRole::Mint => format!(
                    "{} {}",
                    registered_address.label.bright_green().bold(),
                    "[mint]".to_string().dimmed()
                ),
                AddressRole::Ata { .. } => format!(
                    "{} {}",
                    registered_address.label.bright_yellow().bold(),
                    "[ata]".to_string().dimmed()
                ),
                AddressRole::Pda { seeds, .. } => format!(
                    "{} {}",
                    registered_address.label.bright_magenta().bold(),
                    format!("[pda:{}]", seeds.first().unwrap_or(&"".to_string())).dimmed()
                ),
                AddressRole::Program => format!(
                    "{} {}",
                    registered_address.label.bright_blue().bold(),
                    "[program]".to_string().dimmed()
                ),
                AddressRole::Custom(role) => format!(
                    "{} {}",
                    registered_address.label.bright_white().bold(),
                    format!("[{}]", role).dimmed()
                ),
            },
            None => format!("{}", pubkey.to_string().bright_red()),
        }
    }

    /// Replace all pubkey addresses in a string with their labels from the address book
    pub fn replace_addresses_in_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Sort by pubkey string length (longest first) to avoid partial replacements
        let mut sorted_addresses: Vec<_> = self.addresses.iter().collect();
        sorted_addresses.sort_by_key(|(pubkey, _)| std::cmp::Reverse(pubkey.to_string().len()));

        for (pubkey, registered_addresses) in sorted_addresses {
            if let Some(registered_address) = registered_addresses.first() {
                let pubkey_str = pubkey.to_string();
                let replacement = match &registered_address.role {
                    AddressRole::Wallet => {
                        format!("{}", registered_address.label.bright_cyan().bold())
                    }
                    AddressRole::Mint => {
                        format!("{}", registered_address.label.bright_green().bold())
                    }
                    AddressRole::Ata { .. } => {
                        format!("{}", registered_address.label.bright_yellow().bold())
                    }
                    AddressRole::Pda { .. } => {
                        format!("{}", registered_address.label.bright_magenta().bold())
                    }
                    AddressRole::Program => {
                        format!("{}", registered_address.label.bright_blue().bold())
                    }
                    AddressRole::Custom(_) => {
                        format!("{}", registered_address.label.bright_white().bold())
                    }
                };
                result = result.replace(&pubkey_str, &replacement);
            }
        }

        result
    }

    /// Print all addresses in the address book with colors
    pub fn print_all(&self) {
        if self.addresses.is_empty() {
            println!("📖 Address book is empty");
            return;
        }

        println!("\n{}", "═".repeat(80).dimmed());
        println!(
            "📖 {} ({} entries):",
            "Address Book".bold(),
            self.addresses.len()
        );
        println!("{}", "─".repeat(80).dimmed());

        // Group by role type
        let mut wallets = Vec::new();
        let mut mints = Vec::new();
        let mut atas = Vec::new();
        let mut pdas = Vec::new();
        let mut programs = Vec::new();
        let mut custom = Vec::new();

        for (pubkey, regs) in &self.addresses {
            for reg in regs {
                match &reg.role {
                    AddressRole::Wallet => wallets.push((pubkey, reg)),
                    AddressRole::Mint => mints.push((pubkey, reg)),
                    AddressRole::Ata { .. } => atas.push((pubkey, reg)),
                    AddressRole::Pda { .. } => pdas.push((pubkey, reg)),
                    AddressRole::Program => programs.push((pubkey, reg)),
                    AddressRole::Custom(_) => custom.push((pubkey, reg)),
                }
            }
        }

        // Print each category
        if !programs.is_empty() {
            println!(
                "\n  {} {}:",
                "Programs".bright_blue().bold(),
                format!("({})", programs.len()).dimmed()
            );
            for (pubkey, reg) in programs {
                println!(
                    "    {} {}",
                    "•".to_string().bright_blue(),
                    format!(
                        "{:<30} {}",
                        reg.label.bright_blue().bold(),
                        pubkey.to_string().dimmed()
                    )
                );
            }
        }

        if !wallets.is_empty() {
            println!(
                "\n  {} {}:",
                "Wallets".bright_cyan().bold(),
                format!("({})", wallets.len()).dimmed()
            );
            for (pubkey, reg) in wallets {
                println!(
                    "    {} {}",
                    "•".to_string().bright_cyan(),
                    format!(
                        "{:<30} {}",
                        reg.label.bright_cyan().bold(),
                        pubkey.to_string().dimmed()
                    )
                );
            }
        }

        if !mints.is_empty() {
            println!(
                "\n  {} {}:",
                "Mints".bright_green().bold(),
                format!("({})", mints.len()).dimmed()
            );
            for (pubkey, reg) in mints {
                println!(
                    "    {} {}",
                    "•".to_string().bright_green(),
                    format!(
                        "{:<30} {}",
                        reg.label.bright_green().bold(),
                        pubkey.to_string().dimmed()
                    )
                );
            }
        }

        if !pdas.is_empty() {
            println!(
                "\n  {} {}:",
                "PDAs".bright_magenta().bold(),
                format!("({})", pdas.len()).dimmed()
            );
            for (pubkey, reg) in pdas {
                if let AddressRole::Pda { seeds, .. } = &reg.role {
                    println!(
                        "    {} {}",
                        "•".to_string().bright_magenta(),
                        format!(
                            "{:<30} {} {}",
                            reg.label.to_string().bright_magenta().bold(),
                            pubkey.to_string().dimmed(),
                            format!("[{}]", seeds.join(",")).dimmed()
                        )
                    );
                }
            }
        }

        if !atas.is_empty() {
            println!(
                "\n  {} {}:",
                "ATAs".bright_yellow().bold(),
                format!("({})", atas.len()).dimmed()
            );
            for (pubkey, reg) in atas {
                println!(
                    "    {} {}",
                    "•".to_string().bright_yellow(),
                    format!(
                        "{:<30} {}",
                        reg.label.bright_yellow().bold(),
                        pubkey.to_string().dimmed()
                    )
                );
            }
        }

        if !custom.is_empty() {
            println!(
                "\n  {} {}:",
                "Custom".bright_white().bold(),
                format!("({})", custom.len()).dimmed()
            );
            for (pubkey, reg) in custom {
                if let AddressRole::Custom(role) = &reg.role {
                    println!(
                        "    {} {}",
                        "•".to_string().bright_white(),
                        format!(
                            "{:<30} {} {}",
                            reg.label.bright_white().bold(),
                            pubkey.to_string().dimmed(),
                            format!("[{}]", role).dimmed()
                        )
                    );
                }
            }
        }

        println!("{}", "═".repeat(80).dimmed());
    }

    /// Check if an address exists in the book
    pub fn contains(&self, pubkey: &Pubkey) -> bool {
        self.addresses.contains_key(pubkey)
    }

    /// Get the number of entries in the address book
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Check if the address book is empty
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_book_new() {
        let book = AddressBook::new();
        assert_eq!(book.len(), 0);
        assert!(book.is_empty());
    }

    #[test]
    fn test_add_wallet() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();

        book.add_wallet(pubkey, "test_wallet".to_string()).unwrap();

        assert_eq!(book.len(), 1);
        assert!(book.contains(&pubkey));
        assert_eq!(book.get_label(&pubkey), "test_wallet");
    }

    #[test]
    fn test_add_mint() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();

        book.add_mint(pubkey, "test_mint".to_string()).unwrap();

        let registered = book.get_first(&pubkey).unwrap();
        assert_eq!(registered.label, "test_mint");
        matches!(registered.role, AddressRole::Mint);
    }

    #[test]
    fn test_add_ata() {
        let mut book = AddressBook::new();
        let ata_pubkey = Pubkey::new_unique();
        let mint_pubkey = Pubkey::new_unique();
        let owner_pubkey = Pubkey::new_unique();

        book.add_ata(
            ata_pubkey,
            "test_ata".to_string(),
            mint_pubkey,
            owner_pubkey,
        )
        .unwrap();

        let registered = book.get_first(&ata_pubkey).unwrap();
        assert_eq!(registered.label, "test_ata");
        if let AddressRole::Ata { mint, owner } = &registered.role {
            assert_eq!(*mint, mint_pubkey);
            assert_eq!(*owner, owner_pubkey);
        } else {
            panic!("Expected ATA role");
        }
    }

    #[test]
    fn test_add_program() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();

        book.add_program(pubkey, "test_program").unwrap();

        let registered = book.get_first(&pubkey).unwrap();
        assert_eq!(registered.label, "test_program");
        matches!(registered.role, AddressRole::Program);
    }

    #[test]
    fn test_add_custom() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();

        book.add_custom(
            pubkey,
            "test_custom".to_string(),
            "special_role".to_string(),
        )
        .unwrap();

        let registered = book.get_first(&pubkey).unwrap();
        assert_eq!(registered.label, "test_custom");
        if let AddressRole::Custom(role) = &registered.role {
            assert_eq!(role, "special_role");
        } else {
            panic!("Expected Custom role");
        }
    }

    #[test]
    fn test_duplicate_label_error() {
        let mut book = AddressBook::new();
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();

        book.add_wallet(pubkey1, "duplicate".to_string()).unwrap();

        let result = book.add_wallet(pubkey2, "duplicate".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_get_by_role() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();

        book.add_wallet(pubkey, "test_wallet".to_string()).unwrap();

        let found = book.get_by_role(&AddressRole::Wallet);
        assert_eq!(found, Some(pubkey));

        let not_found = book.get_by_role(&AddressRole::Mint);
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_get_all_by_role_type() {
        let mut book = AddressBook::new();
        let wallet1 = Pubkey::new_unique();
        let wallet2 = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        book.add_wallet(wallet1, "wallet1".to_string()).unwrap();
        book.add_wallet(wallet2, "wallet2".to_string()).unwrap();
        book.add_mint(mint, "mint1".to_string()).unwrap();

        let wallets = book.get_all_by_role_type("wallet");
        assert_eq!(wallets.len(), 2);
        assert!(wallets.contains(&wallet1));
        assert!(wallets.contains(&wallet2));

        let mints = book.get_all_by_role_type("mint");
        assert_eq!(mints.len(), 1);
        assert!(mints.contains(&mint));
    }

    #[test]
    fn test_pda_creation() {
        let program_id = Pubkey::new_unique();
        let seeds = vec!["test", "seed"];

        let (_pubkey, bump, registered) = RegisteredAddress::pda("test_pda", &seeds, &program_id);

        assert_eq!(registered.label, "test_pda");
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

    #[test]
    fn test_format_address() {
        let mut book = AddressBook::new();
        let pubkey = Pubkey::new_unique();
        let unknown_pubkey = Pubkey::new_unique();

        book.add_wallet(pubkey, "test_wallet".to_string()).unwrap();

        let formatted = book.format_address(&pubkey);
        assert!(formatted.contains("test_wallet"));

        let unknown_formatted = book.format_address(&unknown_pubkey);
        assert!(unknown_formatted.contains(&unknown_pubkey.to_string()));
    }
}
