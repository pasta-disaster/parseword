// src/cli/handlers.rs

use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::args::{FieldArg, OutputFormat};
use crate::cli::dto::{
    EntryCreatedDto, EntryDetailDto, EntrySummaryDto, FieldDto, VaultInitDto, VaultInspectDto,
};
use crate::core::entry::Entry;
use crate::core::field::{Field, FieldValue};
use crate::core::persistence::{decrypt_vault, encrypt_vault, load_from_file, save_to_file};
use crate::core::vault::Vault;

// helpers

fn with_unlocked_vault<F, T>(path: &PathBuf, operation: F) -> Result<T, String>
where
    F: FnOnce(&Vault) -> Result<T, String>,
{
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let passphrase = rpassword::prompt_password("Enter master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;

    let encrypted = load_from_file(path)?;
    let vault = decrypt_vault(&encrypted, &passphrase)?;

    let output = operation(&vault)?;

    Ok(output)
}

fn with_unlocked_vault_mut<F, T>(path: &PathBuf, operation: F) -> Result<T, String>
where
    F: FnOnce(&mut Vault) -> Result<T, String>,
{
    if !path.exists() {
        return Err(format!("Vault file not found at path: {}", path.display()));
    }

    let passphrase = rpassword::prompt_password("Enter master passphrase: ")
        .map_err(|e| format!("Failed to read passphrase: {}", e))?;

    let encrypted = load_from_file(path)?;
    let mut vault = decrypt_vault(&encrypted, &passphrase)?;

    let output = operation(&mut vault)?;

    let updated_encrypted = encrypt_vault(&vault, &passphrase)?;
    save_to_file(path, &updated_encrypted)?;

    Ok(output)
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

pub fn render_output<T, F>(format: OutputFormat, data: &T, text_renderer: F) -> Result<(), String>
where
    T: Serialize,
    F: FnOnce() -> Result<(), String>,
{
    match format {
        OutputFormat::Text => text_renderer(),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(data)
                .map_err(|e| format!("Failed to serialise output to JSON: {}", e))?;
            println!("{}", json);
            Ok(())
        }
        OutputFormat::Yaml => {
            let yaml = serde_yml::to_string(data)
                .map_err(|e| format!("Failed to serialise output to YAML: {}", e))?;
            println!("{}", yaml);
            Ok(())
        }
    }
}

// handlers

pub fn handle_init(path: PathBuf, name: String, format: OutputFormat) -> Result<(), String> {
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

    let dto = VaultInitDto {
        id: &vault.id,
        name: &vault.name,
        path: path.display().to_string(),
    };

    render_output(format, &dto, || {
        println!("Initialising new vault: '{}'", dto.name);
        println!("Vault successfully encrypted and saved to: {}", dto.path);
        Ok(())
    })
}

pub fn handle_inspect(path: PathBuf, format: OutputFormat) -> Result<(), String> {
    with_unlocked_vault(&path, |vault| {
        let dto = VaultInspectDto {
            id: &vault.id,
            name: &vault.name,
            is_locked: vault.is_locked,
            entry_count: vault.entries().map_or(0, |e| e.len()),
        };

        render_output(format, &dto, || {
            println!("\n=== Vault unlocked successfully ===");
            println!("ID:       {}", vault.id);
            println!("Name:     {}", vault.name);
            println!("Locked:   {}", vault.is_locked);
            println!("Entries:  {}", vault.entries().map_or(0, |e| e.len()));
            println!("=====================================");

            Ok(())
        })
    })
}

pub fn handle_list(path: PathBuf, format: OutputFormat) -> Result<(), String> {
    with_unlocked_vault(&path, |vault| {
        let entries = vault.entries().unwrap_or(&[]);

        let dtos: Vec<EntrySummaryDto> = entries
            .iter()
            .map(|e| EntrySummaryDto {
                id: &e.id,
                title: &e.title,
                field_count: e.fields.len(),
            })
            .collect();

        render_output(format, &dtos, || {
            if entries.is_empty() {
                println!("Vault contains no entries.");
                return Ok(());
            }

            println!("\n=== Vault Entries ({}) ===", entries.len());
            for (i, entry) in entries.iter().enumerate() {
                println!("  {:2}, {}", i + 1, entry.title);
            }
            Ok(())
        })
    })
}

pub fn handle_add_entry(path: PathBuf, title: String, format: OutputFormat) -> Result<(), String> {
    if title.is_empty() {
        return Err("You must provide a title for the entry".to_string());
    }

    let entry_id = uuid_v4_simple();
    let entry_title = title.clone();

    with_unlocked_vault_mut(&path, |vault| {
        let entry = Entry::new(entry_id.clone(), entry_title.clone(), vec![]);

        vault.add_entry(entry)?;

        Ok(())
    })?;

    let dto = EntryCreatedDto {
        id: &entry_id,
        title: &entry_title,
    };

    render_output(format, &dto, || {
        println!("Entry successfully created: {}", title);

        Ok(())
    })
}

pub fn handle_add_field(
    path: PathBuf,
    entry: String,
    field_args: Vec<FieldArg>,
    format: OutputFormat,
) -> Result<(), String> {
    if entry.is_empty() {
        return Err("You must provide an entry to add the field(s) to".to_string());
    }

    if field_args.is_empty() {
        return Err("You must provide at least one --field argument".to_string());
    }

    let is_interactive_terminal = std::io::stdin().is_terminal();
    if format != OutputFormat::Text {
        for arg in &field_args {
            if arg.value.is_none() {
                return Err(format!(
                    "Field '{}' is missing a value. Non-text output formats require all field values.",
                    arg.label
                ));
            }
        }
    }

    let mut resolved_fields = Vec::new();

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

        resolved_fields.push(Field::new(arg.label, value));
    }

    let entry_title = entry.clone();
    let updated_entry_data = with_unlocked_vault_mut(&path, |vault| {
        let target_entry = vault
            .get_entry_mut(&entry)
            .ok_or_else(|| format!("Entry not found: {}", entry))?;

        for field in resolved_fields {
            target_entry.add_field(field);
        }

        Ok((
            target_entry.id.clone(),
            target_entry.title.clone(),
            target_entry.fields.clone(),
        ))
    })?;

    let (entry_id, title, fields) = updated_entry_data;

    let dto = EntryDetailDto {
        id: &entry_id,
        title: &title,
        fields: fields.iter().map(FieldDto::from).collect(),
    };

    render_output(format, &dto, || {
        println!("Successfully updated entry '{}'.", entry);
        Ok(())
    })
}

pub fn handle_view(path: PathBuf, entry: String, format: OutputFormat) -> Result<(), String> {
    if entry.is_empty() {
        return Err("You must provide an entry".to_string());
    }

    with_unlocked_vault(&path, |vault| {
        let target_entry = vault
            .get_entry(&entry)
            .ok_or_else(|| format!("Entry not found: {}", entry))?;

        let dto = EntryDetailDto {
            id: &target_entry.id,
            title: &target_entry.title,
            fields: target_entry.fields.iter().map(FieldDto::from).collect(),
        };

        render_output(format, &dto, || {
            println!("{}", target_entry);
            Ok(())
        })
    })
}

pub fn handle_dump(path: PathBuf) -> Result<(), String> {
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
