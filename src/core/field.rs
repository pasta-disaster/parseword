// src/core/field.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FieldType {
    Username,
    Password,
    Url,
    Note,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(String),
    Password(String),
    File { filename: String, bytes: Vec<u8> },
    Note(String),
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Text(s) => write!(f, "{}", s),
            FieldValue::Password(_) => write!(f, "***"),
            FieldValue::File { filename, bytes } => write!(f, "{}", filename),
            FieldValue::Note(s) => write!(f, "{}", s),
        }
    }
}

impl Zeroize for FieldValue {
    fn zeroize(&mut self) {
        match self {
            FieldValue::Text(s) => s.zeroize(),
            FieldValue::Password(s) => s.zeroize(),
            FieldValue::File { filename, bytes } => {
                filename.zeroize();
                bytes.zeroize();
            }
            FieldValue::Note(s) => s.zeroize(),
        }
    }
}

// automatically trigger zeroize when variable goes out of scope
impl ZeroizeOnDrop for FieldValue {}

#[derive(Serialize, Deserialize, Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Field {
    //    pub id: String,
    pub label: String,
    pub value: FieldValue,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.label, self.value)
    }
}

impl Field {
    pub fn new(label: impl Into<String>, value: FieldValue) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }

    pub fn fmt_padded(&self, f: &mut Formatter<'_>, label_width: usize) -> fmt::Result {
        let value_str = match &self.value {
            FieldValue::Text(v) => v.clone(),
            FieldValue::Password(_) => "*****".to_string(),
            FieldValue::Note(v) => v.clone(),
            FieldValue::File { filename, bytes } => {
                format!("file {} ({} bytes)", filename, bytes.len())
            }
        };

        writeln!(
            f,
            "  {:<width$} = {}",
            self.label,
            value_str,
            width = label_width
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_creation() {
        let field = Field::new("Password", FieldValue::Password("Test Value".into()));
        assert_eq!(field.label, "Password");
        assert_eq!(field.value, FieldValue::Password("Test Value".into()));
    }
}
