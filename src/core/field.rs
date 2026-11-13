// src/core/field.rs

use serde::{Deserialize, Serialize};
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

impl Field {
    pub fn new(label: impl Into<String>, value: FieldValue) -> Self {
        Self {
            label: label.into(),
            value,
        }
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
