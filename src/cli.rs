// src/cli.rs

use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

use crate::core::persistence::{decrypt_vault, encrypt_vault, load_from_file, save_to_file};
use crate::core::vault::Vault;

#[derive(Parser)]
#[command(name = "parseword")]
#[command(about = "A secure, local Rust password manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, name } => handle_init(path, name),
        Commands::Inspect { path } => handle_inspect(path),
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
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let encrypted = load_from_file(&path)?;

    let passphrase = rpassword::prompt_password("Enter master passphrase to unlock")
fn with_unlocked_vault<F>(path: &PathBuf, operation: F) -> Result<(), String>
where
    F: FnOnce(&Vault) -> Result<(), String>,
{
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let passphrase = rpassword::prompt_password("Enter master passphrase")
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

    let passphrase = rpassword::prompt_password("Enter master passphrase")
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
