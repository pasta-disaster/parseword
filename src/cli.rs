// src/cli.rs

use clap::{Parser, Subcommand};
use std::io::IsTerminal;
use std::io::Write;
use std::path::PathBuf;

use std::slice::SplitN;
use std::str::FromStr;

use crate::core::entry::Entry;
use crate::core::field::{self, Field, FieldValue};
use crate::core::persistence::{decrypt_vault, encrypt_vault, load_from_file, save_to_file};
use crate::core::vault::Vault;

#[derive(Parser)]
#[command(name = "parseword")]
#[command(about = "A secure, local Rust password manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone)]
pub struct FieldArg {
    pub field_type: String,
    pub label: String,
    pub value: Option<String>,
}

impl FromStr for FieldArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut field_type = None;
        let mut label = None;
        let mut value = None;

        for part in s.split(',') {
            let mut kv = part.splitn(2, "=");
            let k = kv
                .next()
                .ok_or_else(|| format!("Invalid field format: {}", s))?
                .trim();
            let v = kv.next().map(|val| val.trim().to_string());

            match k {
                "type" => field_type = v,
                "label" => label = v,
                "value" => value = v,
                _ => return Err(format!("Unknown field parameter: {}", k)),
            }
        }

        Ok(FieldArg {
            field_type: field_type.ok_or("Missing 'type' parameter")?,
            label: label.ok_or("Missing 'label' parameter")?,
            value,
        })
    }
}

#[derive(Subcommand)]
enum Commands {
    // initialise and save a new encrypted vault file
    Init {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
        #[arg(short, long, default_value = "My Vault")]
        name: String,
    },
    // load, decrypt and inspect an existing vault file
    Inspect {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
    },
    List {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
    },
    // add an entry to a vault
    AddEntry {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
        #[arg(short, long)]
        title: String,
    },
    AddField {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
        #[arg(short, long)]
        entry: String,
        #[arg(short, long, value_name = "TYPE=...,LABEL=...,VALUE=...")]
        field_args: Vec<FieldArg>,
    },
    View {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
        #[arg(short, long)]
        entry: String,
    },
    Dump {
        #[arg(short, long, default_value = "vault.pwv")]
        path: PathBuf,
    },
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, name } => handle_init(path, name),
        Commands::Inspect { path } => handle_inspect(path),
        Commands::List { path } => handle_list(path),
        Commands::AddEntry { path, title } => handle_add_entry(path, title),
        Commands::AddField {
            path,
            entry,
            field_args,
        } => handle_add_field(path, entry, field_args),
        Commands::View { path, entry } => handle_view(path, entry),
        Commands::Dump { path } => handle_dump(path),
    }
}

fn handle_init(path: PathBuf, name: String) -> Result<(), String> {
    println!("Initializing new vault: '{}'", name);

    // prompt for master passphrase secure
    let passphrase = rpassword::prompt_password("Enter master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;
    let confirm = rpassword::prompt_password("Repeat master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;

    if passphrase != confirm {
        return Err("Passphrases to not match. Aborting.".to_string());
    }

    if passphrase.is_empty() {
        return Err("Passphrase cannot be empty.".to_string());
    }

    let vault_id = uuid_v4_simple();
    let mut vault = Vault::new(vault_id, name);

    // unlock vault so that it has an initialised entry state to persist
    vault.unlock(vec![]);

    // encrypt and serialise to disk
    let encrypted = encrypt_vault(&vault, &passphrase)?;
    save_to_file(&path, &encrypted)?;

    println!(
        "Vault successfully encrypted and saved to: {}",
        path.display()
    );

    Ok(())
}

fn handle_inspect(path: PathBuf) -> Result<(), String> {
    with_unlocked_vault(&path, |vault| {
        println!("\n=== Vault unlocked successfully ===");
        println!("ID:       {}", vault.id);
        println!("Name:     {}", vault.name);
        println!("Locked:   {}", vault.is_locked);
        println!("Entries:  {}", vault.entries().map_or(0, |e| e.len()));
        println!("=====================================");

        Ok(())
    })
}

fn handle_list(path: PathBuf) -> Result<(), String> {
    with_unlocked_vault(&path, |vault| {
        let entries = vault
            .entries()
            .ok_or("Vault is locked or has no entries".to_string())?;

        for e in entries {
            println!("  {}", e.title);
        }

        Ok(())
    })
}

fn handle_add_entry(path: PathBuf, title: String) -> Result<(), String> {
    if title.is_empty() {
        return Err("You must provide a title for the entry".to_string());
    }

    with_unlocked_vault_mut(&path, |vault| {
        let entry = Entry::new(uuid_v4_simple(), title.clone(), vec![]);
        vault.add_entry(entry)?;

        println!("Entry successfully created: {}", title);

        Ok(())
    })
}

fn handle_add_field(path: PathBuf, entry: String, field_args: Vec<FieldArg>) -> Result<(), String> {
    if entry.is_empty() {
        return Err("You must provide an entry to add the field(s) to".to_string());
    }

    if field_args.is_empty() {
        return Err("You must provide at least one --field argument".to_string());
    }

    with_unlocked_vault_mut(&path, |vault| {
        let target_entry = vault
            .get_entry_mut(&entry)
            .ok_or_else(|| format!("Entry not found: {}", entry))?;

        let is_interactive_terminal = std::io::stdin().is_terminal();

        for arg in field_args {
            let field_type_lower = arg.field_type.to_lowercase();

            if field_type_lower == "password" && arg.value.is_some() {
                eprintln!(
                    "[WARNING] Passing sensitive data like passwords via CLI flags exposes them to shell history and process tables."
                );
            }

            let resolved_value = match arg.value {
                Some(val) => val,
                // abort if values missing on non-interactive shell
                None => {
                    if !is_interactive_terminal {
                        return Err(format!(
                            "Non-interactive environment detected, but value was omitted for field {}. Aborting.",
                            arg.label
                        ));
                    }

                    if field_type_lower == "password" {
                        rpassword::prompt_password(format!("Enter password for {}: ", arg.label))
                            .map_err(|e| format!("Failed to read password {}", e))?
                    } else {
                        print!("Enter value for {}: ", arg.label);
                        std::io::stdout()
                            .flush()
                            .map_err(|e| format!("Failed to flush stdout: {}", e))?;

                        let mut input = String::new();
                        std::io::stdin()
                            .read_line(&mut input)
                            .map_err(|e| format!("Failed to read value {}: ", e))?;
                        input.trim().to_string()
                    }
                }
            };

            let value = match field_type_lower.as_str() {
                "text" | "t" => FieldValue::Text(resolved_value),
                "password" | "p" => FieldValue::Password(resolved_value),
                "note" | "n" => FieldValue::Note(resolved_value),
                "file" | "f" => {
                    let file_path = std::path::Path::new(&resolved_value);
                    let filename = file_path
                        .file_name()
                        .ok_or_else(|| format!("Invalid file path: {}", resolved_value))?
                        .to_string_lossy()
                        .into_owned();
                    let bytes = std::fs::read(file_path).map_err(|e| {
                        format!("Failed to read file at '{}': {}", file_path.display(), e)
                    })?;
                    FieldValue::File { filename, bytes }
                }
                other => return Err(format!("Unsupported field type: {}", other)),
            };

            target_entry.add_field(Field::new(arg.label, value));
        }

        println!("Successfully updated entry '{}'.", entry);
        Ok(())
    })
}

fn handle_view(path: PathBuf, entry: String) -> Result<(), String> {
    if entry.is_empty() {
        return Err("You must provide an entry".to_string());
    }

    with_unlocked_vault(&path, |vault| {
        let target_entry = vault
            .get_entry(&entry)
            .ok_or_else(|| format!("Entry not found: {}", entry))?;

        println!("{}", target_entry);

        Ok(())
    })
}

fn handle_dump(path: PathBuf) -> Result<(), String> {
    with_unlocked_vault(&path, |vault| {
        let entries = vault
            .entries()
            .ok_or("Vault is locked or has no entries".to_string())?;

        println!("\n=== Vault unlocked successfully ===");
        println!("ID:       {}", vault.id);
        println!("Name:     {}", vault.name);
        println!("Locked:   {}", vault.is_locked);
        println!("Entries:  {}", vault.entries().map_or(0, |e| e.len()));
        for e in entries {
            println!("{}", e);
        }
        println!("=====================================");

        Ok(())
    })
}

fn with_unlocked_vault<F>(path: &PathBuf, operation: F) -> Result<(), String>
where
    F: FnOnce(&Vault) -> Result<(), String>,
{
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let passphrase = rpassword::prompt_password("Enter master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;

    let encrypted = load_from_file(&path)?;
    let vault = decrypt_vault(&encrypted, &passphrase)?;

    operation(&vault)?;

    Ok(())
}

fn with_unlocked_vault_mut<F>(path: &PathBuf, operation: F) -> Result<(), String>
where
    F: FnOnce(&mut Vault) -> Result<(), String>,
{
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let passphrase = rpassword::prompt_password("Enter master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;

    let encrypted = load_from_file(&path)?;
    let mut vault = decrypt_vault(&encrypted, &passphrase)?;

    operation(&mut vault)?;

    let updated_encrypted = encrypt_vault(&vault, &passphrase)?;
    save_to_file(path, &updated_encrypted)?;

    Ok(())
}

fn uuid_v4_simple() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:12x}",
        rng.r#gen::<u32>(),
        rng.r#gen::<u16>(),
        rng.r#gen::<u16>(),
        rng.r#gen::<u16>(),
        rng.r#gen::<u64>() & 0x0000_ffff_ffff_ffff
    )
}
