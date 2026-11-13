// src/core/field.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Password(String),
    File { filename: String, bytes: Vec<u8> },
    Note(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub value: FieldValue,
}
