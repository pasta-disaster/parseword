// src/cli/args.rs

use std::path::PathBuf;
use std::str::FromStr;

use clap::ValueEnum;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "parseword")]
#[command(about = "A secure, local Rust password manager", long_about = None)]
pub struct Cli {
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    pub format: OutputFormat,
    #[command(subcommand)]
    pub command: Commands,
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
pub enum Commands {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}
