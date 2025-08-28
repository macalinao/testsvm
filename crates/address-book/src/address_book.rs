//! Core address book implementation for managing Solana addresses with labels and roles.

use crate::pda_seeds::{SeedPart, find_pda_with_bump_and_strings};
use crate::registered_address::{AddressRole, RegisteredAddress};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anyhow::{Result, anyhow};
use colored::*;
use std::collections::{HashMap, HashSet};

/// Address book for mapping public keys to registered addresses with labels.
///
/// This structure maintains multiple mappings to efficiently track and query
/// Solana addresses by their public keys, labels, and roles. It's designed
/// to help with debugging and transaction analysis by providing meaningful
/// context for addresses.
#[derive(Debug, Default)]
pub struct AddressBook {
    addresses: HashMap<Pubkey, Vec<(String, RegisteredAddress)>>,
    registered_addresses: HashSet<RegisteredAddress>,
    labels: HashMap<String, RegisteredAddress>,
}

impl AddressBook {
    /// Creates a new empty address book.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    ///
    /// let book = AddressBook::new();
    /// assert!(book.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
            registered_addresses: HashSet::new(),
            labels: HashMap::new(),
        }
    }

    /// Adds default Solana system programs to the address book.
    ///
    /// This includes:
    /// - System Program
    /// - Token Program
    /// - Associated Token Program
    ///
    /// # Errors
    ///
    /// Returns an error if any of the default programs fail to be added
    /// (e.g., due to duplicate labels).
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    ///
    /// let mut book = AddressBook::new();
    /// book.add_default_accounts().unwrap();
    ///
    /// // The system program is now registered
    /// assert!(book.contains(&anchor_lang::solana_program::system_program::ID));
    /// ```
    pub fn add_default_accounts(&mut self) -> Result<()> {
        self.add_program(system_program::ID, "system_program")?;
        self.add_program(anchor_spl::token::ID, "token_program")?;
        self.add_program(anchor_spl::associated_token::ID, "associated_token_program")?;
        Ok(())
    }

    /// Gets the label for a given public key.
    ///
    /// If the address is registered, returns its label. Otherwise, returns
    /// the string representation of the public key.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// // Before registration, returns the pubkey string
    /// assert_eq!(book.get_label(&wallet), wallet.to_string());
    ///
    /// // After registration, returns the label
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    /// assert_eq!(book.get_label(&wallet), "alice");
    /// ```
    pub fn get_label(&self, pubkey: &Pubkey) -> String {
        self.addresses
            .get(pubkey)
            .and_then(|v| v.first())
            .map(|(label, _)| label.clone())
            .unwrap_or_else(|| pubkey.to_string())
    }

    /// Adds an address with a registered address and label to the address book.
    ///
    /// This is the core method for adding addresses. All other add methods
    /// (add_wallet, add_mint, etc.) internally use this method.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address or role.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::{AddressBook, RegisteredAddress};
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    /// let registered = RegisteredAddress::wallet(wallet);
    ///
    /// book.add(wallet, "my_wallet".to_string(), registered).unwrap();
    /// assert_eq!(book.get_label(&wallet), "my_wallet");
    /// ```
    pub fn add(
        &mut self,
        pubkey: Pubkey,
        label: String,
        registered_address: RegisteredAddress,
    ) -> Result<()> {
        // Check if this label already exists
        if let Some(existing_address) = self.labels.get(&label) {
            if existing_address.key != pubkey || existing_address.role != registered_address.role {
                return Err(anyhow!("Label '{}' already exists in address book", label));
            }
            return Ok(());
        }

        // Add to labels and registered addresses
        self.labels
            .insert(label.clone(), registered_address.clone());
        self.registered_addresses.insert(registered_address.clone());

        // Add to addresses vector (allows multiple registrations per pubkey)
        self.addresses
            .entry(pubkey)
            .or_default()
            .push((label, registered_address));

        Ok(())
    }

    /// Adds a wallet address to the address book.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    ///
    /// // The wallet is now registered
    /// assert!(book.contains(&wallet));
    /// assert_eq!(book.get_label(&wallet), "alice");
    /// ```
    pub fn add_wallet(&mut self, pubkey: Pubkey, label: String) -> Result<()> {
        self.add(pubkey, label, RegisteredAddress::wallet(pubkey))
    }

    /// Adds a token mint address to the address book.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let mint = Pubkey::new_unique();
    ///
    /// book.add_mint(mint, "usdc_mint".to_string()).unwrap();
    /// assert_eq!(book.get_label(&mint), "usdc_mint");
    /// ```
    pub fn add_mint(&mut self, pubkey: Pubkey, label: String) -> Result<()> {
        self.add(pubkey, label, RegisteredAddress::mint(pubkey))
    }

    /// Adds an Associated Token Account (ATA) address to the address book.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The ATA's public key
    /// * `label` - Human-readable label for the ATA
    /// * `mint` - The token mint public key
    /// * `owner` - The owner's public key
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let ata = Pubkey::new_unique();
    /// let mint = Pubkey::new_unique();
    /// let owner = Pubkey::new_unique();
    ///
    /// book.add_ata(ata, "alice_usdc".to_string(), mint, owner).unwrap();
    /// assert_eq!(book.get_label(&ata), "alice_usdc");
    /// ```
    pub fn add_ata(
        &mut self,
        pubkey: Pubkey,
        label: String,
        mint: Pubkey,
        owner: Pubkey,
    ) -> Result<()> {
        self.add(pubkey, label, RegisteredAddress::ata(pubkey, mint, owner))
    }

    /// Adds a custom role address to the address book.
    ///
    /// Use this for addresses that don't fit into the standard categories.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let address = Pubkey::new_unique();
    ///
    /// book.add_custom(
    ///     address,
    ///     "dao_treasury".to_string(),
    ///     "governance".to_string()
    /// ).unwrap();
    ///
    /// assert_eq!(book.get_label(&address), "dao_treasury");
    /// ```
    pub fn add_custom(&mut self, pubkey: Pubkey, label: String, custom_role: String) -> Result<()> {
        self.add(
            pubkey,
            label,
            RegisteredAddress::custom(pubkey, &custom_role),
        )
    }

    /// Adds a Program Derived Address (PDA) to the address book.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The PDA's public key
    /// * `label` - Human-readable label for the PDA
    /// * `seeds` - The string representations of seeds used to derive the PDA
    /// * `program_id` - The program that owns the PDA
    /// * `bump` - The bump seed used to derive the PDA
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let pda = Pubkey::new_unique();
    /// let program = Pubkey::new_unique();
    ///
    /// book.add_pda(
    ///     pda,
    ///     "vault".to_string(),
    ///     vec!["vault".to_string(), "v1".to_string()],
    ///     program,
    ///     255
    /// ).unwrap();
    ///
    /// assert_eq!(book.get_label(&pda), "vault");
    /// ```
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
            label,
            RegisteredAddress::pda_from_parts(pubkey, seeds, program_id, bump),
        )
    }

    /// Adds a program address to the address book.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let program = Pubkey::new_unique();
    ///
    /// book.add_program(program, "my_program").unwrap();
    /// assert_eq!(book.get_label(&program), "my_program");
    /// ```
    pub fn add_program(&mut self, pubkey: Pubkey, label: &str) -> Result<()> {
        self.add(
            pubkey,
            label.to_string(),
            RegisteredAddress::program(pubkey),
        )
    }

    /// Finds a PDA with bump and adds it to the address book.
    ///
    /// This method derives the PDA from the provided seeds and program ID,
    /// then automatically registers it in the address book.
    ///
    /// # Returns
    ///
    /// A tuple containing the derived PDA public key and bump seed.
    ///
    /// # Errors
    ///
    /// Returns an error if the label already exists with a different address.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::{AddressBook, SeedPart};
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let program = Pubkey::new_unique();
    /// let user = Pubkey::new_unique();
    ///
    /// let (pda, bump) = book.find_pda_with_bump(
    ///     "user_vault",
    ///     &[&"vault" as &dyn SeedPart, &user as &dyn SeedPart],
    ///     program
    /// ).unwrap();
    ///
    /// assert_eq!(book.get_label(&pda), "user_vault");
    /// ```
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

    /// Gets all registered addresses for a public key.
    ///
    /// A single public key can have multiple registrations with different labels.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    ///
    /// let registrations = book.get(&wallet);
    /// assert!(registrations.is_some());
    /// assert_eq!(registrations.unwrap().len(), 1);
    /// ```
    pub fn get(&self, pubkey: &Pubkey) -> Option<&Vec<(String, RegisteredAddress)>> {
        self.addresses.get(pubkey)
    }

    /// Gets the first registered address for a public key.
    ///
    /// Returns the first label and registered address pair if the pubkey exists.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    ///
    /// let (label, reg) = book.get_first(&wallet).unwrap();
    /// assert_eq!(*label, "alice");
    /// ```
    pub fn get_first(&self, pubkey: &Pubkey) -> Option<(&String, &RegisteredAddress)> {
        self.addresses
            .get(pubkey)
            .and_then(|v| v.first())
            .map(|(label, reg)| (label, reg))
    }

    /// Finds an address by its role.
    ///
    /// Returns the first address that matches the specified role.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::{AddressBook, AddressRole};
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    ///
    /// let found = book.get_by_role(&AddressRole::Wallet);
    /// assert_eq!(found, Some(wallet));
    /// ```
    pub fn get_by_role(&self, role: &AddressRole) -> Option<Pubkey> {
        for registered in self.registered_addresses.iter() {
            if &registered.role == role {
                return Some(registered.key);
            }
        }
        None
    }

    /// Gets all addresses with a specific role type.
    ///
    /// Role types are: "wallet", "mint", "ata", "pda", "program", "custom"
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet1 = Pubkey::new_unique();
    /// let wallet2 = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet1, "alice".to_string()).unwrap();
    /// book.add_wallet(wallet2, "bob".to_string()).unwrap();
    ///
    /// let wallets = book.get_all_by_role_type("wallet");
    /// assert_eq!(wallets.len(), 2);
    /// assert!(wallets.contains(&wallet1));
    /// assert!(wallets.contains(&wallet2));
    /// ```
    pub fn get_all_by_role_type(&self, role_type: &str) -> Vec<Pubkey> {
        let mut addresses = Vec::new();
        for registered in self.registered_addresses.iter() {
            match (&registered.role, role_type) {
                (AddressRole::Wallet, "wallet")
                | (AddressRole::Mint, "mint")
                | (AddressRole::Program, "program") => {
                    addresses.push(registered.key);
                }
                (AddressRole::Ata { .. }, "ata") => {
                    addresses.push(registered.key);
                }
                (AddressRole::Pda { .. }, "pda") => {
                    addresses.push(registered.key);
                }
                (AddressRole::Custom(_), "custom") => {
                    addresses.push(registered.key);
                }
                _ => {}
            }
        }
        addresses
    }

    /// Gets a formatted string representation of an address with colors.
    ///
    /// If the address is registered, returns a colored label with its role.
    /// Otherwise, returns the address string in red.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    ///
    /// let formatted = book.format_address(&wallet);
    /// assert!(formatted.contains("alice"));
    /// assert!(formatted.contains("[wallet]"));
    /// ```
    pub fn format_address(&self, pubkey: &Pubkey) -> String {
        match self.get_first(pubkey) {
            Some((label, registered_address)) => match &registered_address.role {
                AddressRole::Wallet => format!(
                    "{} {}",
                    label.bright_cyan().bold(),
                    "[wallet]".to_string().dimmed()
                ),
                AddressRole::Mint => format!(
                    "{} {}",
                    label.bright_green().bold(),
                    "[mint]".to_string().dimmed()
                ),
                AddressRole::Ata { .. } => format!(
                    "{} {}",
                    label.bright_yellow().bold(),
                    "[ata]".to_string().dimmed()
                ),
                AddressRole::Pda { seeds, .. } => format!(
                    "{} {}",
                    label.bright_magenta().bold(),
                    format!("[pda:{}]", seeds.first().unwrap_or(&"".to_string())).dimmed()
                ),
                AddressRole::Program => format!(
                    "{} {}",
                    label.bright_blue().bold(),
                    "[program]".to_string().dimmed()
                ),
                AddressRole::Custom(role) => format!(
                    "{} {}",
                    label.bright_white().bold(),
                    format!("[{role}]").dimmed()
                ),
            },
            None => format!("{}", pubkey.to_string().bright_red()),
        }
    }

    /// Replaces all public key addresses in text with their labels.
    ///
    /// Scans the provided text for any registered public keys and replaces
    /// them with their colored labels.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let token = Pubkey::new_unique();
    ///
    /// book.add_mint(token, "my_token".to_string()).unwrap();
    ///
    /// let text = format!("Transfer to {}", token);
    /// let formatted = book.replace_addresses_in_text(&text);
    ///
    /// // The pubkey is replaced with the colored label
    /// assert!(formatted.contains("my_token"));
    /// assert!(!formatted.contains(&token.to_string()));
    /// ```
    pub fn replace_addresses_in_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Sort by pubkey string length (longest first) to avoid partial replacements
        let mut sorted_addresses: Vec<_> = self.addresses.iter().collect();
        sorted_addresses.sort_by_key(|(pubkey, _)| std::cmp::Reverse(pubkey.to_string().len()));

        for (pubkey, registered_addresses) in sorted_addresses {
            if let Some((label, registered_address)) = registered_addresses.first() {
                let pubkey_str = pubkey.to_string();
                let replacement = match &registered_address.role {
                    AddressRole::Wallet => format!("{}", label.bright_cyan().bold()),
                    AddressRole::Mint => format!("{}", label.bright_green().bold()),
                    AddressRole::Ata { .. } => format!("{}", label.bright_yellow().bold()),
                    AddressRole::Pda { .. } => format!("{}", label.bright_magenta().bold()),
                    AddressRole::Program => format!("{}", label.bright_blue().bold()),
                    AddressRole::Custom(_) => format!("{}", label.bright_white().bold()),
                };
                result = result.replace(&pubkey_str, &replacement);
            }
        }

        result
    }

    /// Prints all addresses in the address book with colored formatting.
    ///
    /// Addresses are grouped by role type and displayed with appropriate colors.
    /// This is useful for debugging and getting an overview of all registered addresses.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// book.add_wallet(Pubkey::new_unique(), "alice".to_string()).unwrap();
    /// book.add_mint(Pubkey::new_unique(), "usdc".to_string()).unwrap();
    ///
    /// // Prints a formatted table of all addresses
    /// book.print_all();
    /// ```
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
            for (label, reg) in regs {
                match &reg.role {
                    AddressRole::Wallet => wallets.push((pubkey, label, reg)),
                    AddressRole::Mint => mints.push((pubkey, label, reg)),
                    AddressRole::Ata { .. } => atas.push((pubkey, label, reg)),
                    AddressRole::Pda { .. } => pdas.push((pubkey, label, reg)),
                    AddressRole::Program => programs.push((pubkey, label, reg)),
                    AddressRole::Custom(_) => custom.push((pubkey, label, reg)),
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
            for (pubkey, label, _reg) in programs {
                println!(
                    "    {} {:<30} {}",
                    "•".to_string().bright_blue(),
                    label.bright_blue().bold(),
                    pubkey.to_string().dimmed()
                );
            }
        }

        if !wallets.is_empty() {
            println!(
                "\n  {} {}:",
                "Wallets".bright_cyan().bold(),
                format!("({})", wallets.len()).dimmed()
            );
            for (pubkey, label, _reg) in wallets {
                println!(
                    "    {} {:<30} {}",
                    "•".to_string().bright_cyan(),
                    label.bright_cyan().bold(),
                    pubkey.to_string().dimmed()
                );
            }
        }

        if !mints.is_empty() {
            println!(
                "\n  {} {}:",
                "Mints".bright_green().bold(),
                format!("({})", mints.len()).dimmed()
            );
            for (pubkey, label, _reg) in mints {
                println!(
                    "    {} {:<30} {}",
                    "•".to_string().bright_green(),
                    label.bright_green().bold(),
                    pubkey.to_string().dimmed()
                );
            }
        }

        if !pdas.is_empty() {
            println!(
                "\n  {} {}:",
                "PDAs".bright_magenta().bold(),
                format!("({})", pdas.len()).dimmed()
            );
            for (pubkey, label, reg) in pdas {
                if let AddressRole::Pda { seeds, .. } = &reg.role {
                    println!(
                        "    {} {:<30} {} [{}]",
                        "•".to_string().bright_magenta(),
                        label.to_string().bright_magenta().bold(),
                        pubkey.to_string().dimmed(),
                        seeds.join(",").dimmed()
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
            for (pubkey, label, _reg) in atas {
                println!(
                    "    {} {:<30} {}",
                    "•".to_string().bright_yellow(),
                    label.bright_yellow().bold(),
                    pubkey.to_string().dimmed()
                );
            }
        }

        if !custom.is_empty() {
            println!(
                "\n  {} {}:",
                "Custom".bright_white().bold(),
                format!("({})", custom.len()).dimmed()
            );
            for (pubkey, label, reg) in custom {
                if let AddressRole::Custom(role) = &reg.role {
                    println!(
                        "    {} {:<30} {} [{}]",
                        "•".to_string().bright_white(),
                        label.bright_white().bold(),
                        pubkey.to_string().dimmed(),
                        role.dimmed()
                    );
                }
            }
        }

        println!("{}", "═".repeat(80).dimmed());
    }

    /// Checks if an address exists in the book.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// let wallet = Pubkey::new_unique();
    ///
    /// assert!(!book.contains(&wallet));
    ///
    /// book.add_wallet(wallet, "alice".to_string()).unwrap();
    /// assert!(book.contains(&wallet));
    /// ```
    pub fn contains(&self, pubkey: &Pubkey) -> bool {
        self.addresses.contains_key(pubkey)
    }

    /// Returns the number of unique public keys in the address book.
    ///
    /// Note: This counts unique public keys, not total registrations.
    /// A single pubkey can have multiple registrations with different labels.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// assert_eq!(book.len(), 0);
    ///
    /// book.add_wallet(Pubkey::new_unique(), "alice".to_string()).unwrap();
    /// assert_eq!(book.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Checks if the address book is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use address_book::AddressBook;
    /// use anchor_lang::prelude::*;
    ///
    /// let mut book = AddressBook::new();
    /// assert!(book.is_empty());
    ///
    /// book.add_wallet(Pubkey::new_unique(), "alice".to_string()).unwrap();
    /// assert!(!book.is_empty());
    /// ```
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

        let (label, registered) = book.get_first(&pubkey).unwrap();
        assert_eq!(*label, "test_mint");
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

        let (label, registered) = book.get_first(&ata_pubkey).unwrap();
        assert_eq!(*label, "test_ata");
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

        let (label, registered) = book.get_first(&pubkey).unwrap();
        assert_eq!(*label, "test_program");
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

        let (label, registered) = book.get_first(&pubkey).unwrap();
        assert_eq!(*label, "test_custom");
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
        let seeds: Vec<&dyn SeedPart> = vec![&"test", &"seed"];

        let (_pubkey, bump, registered) = RegisteredAddress::pda(&seeds, &program_id);

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
