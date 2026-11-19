// src/core/entry.rs

use super::field::Field;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Entry: {} ===", self.title)?;

        if self.fields.is_empty() {
            writeln!(f, "(no fields)")?;
            return Ok(());
        }

        let max_len = self
            .fields
            .iter()
            .map(|field| field.label.len())
            .max()
            .unwrap_or(0);

        for field in &self.fields {
            field.fmt_padded(f, max_len)?;
        }

        Ok(())
    }
}

impl Entry {
    pub fn new(id: impl Into<String>, title: impl Into<String>, fields: Vec<Field>) -> Self {
        let now = current_timestamp();

        Self {
            id: id.into(),
            title: title.into(),
            fields,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
        self.updated_at = current_timestamp();
    }

    pub fn find_field_mut(&mut self, label: &str) -> Option<&mut Field> {
        self.fields.iter_mut().find(|f| f.label == label)
    }

    pub fn find_field(&self, label: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.label == label)
    }

    pub fn remove_field(&mut self, label: &str) -> Result<Field, String> {
        let pos = self
            .fields
            .iter()
            .position(|f| f.label == label)
            .ok_or_else(|| format!("Field not found: {}", label))?;

        let removed = self.fields.remove(pos);
        self.updated_at = current_timestamp();

        Ok(removed)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::field::FieldValue;

    #[test]
    fn test_find_field() {
        let mut entry = Entry::new("test-entry-id", "Test Entry", vec![]);
        entry.add_field(Field::new(
            "Test Field",
            FieldValue::Text("Test field value".into()),
        ));

        assert!(entry.find_field("Test Field").is_some());
    }

    #[test]
    fn test_add_field() {
        let mut entry = Entry::new("test-entry-id", "Test Entry", vec![]);
        assert_eq!(entry.fields.len(), 0);

        entry.add_field(Field::new(
            "Test Field",
            FieldValue::Text("Test field value".into()),
        ));

        assert_eq!(entry.fields.len(), 1);
        assert!(entry.find_field("Test Field").is_some());
    }

    #[test]
    fn test_remove_field() {
        let mut entry = Entry::new("test-entry-id", "Test Entry", vec![]);
        entry.add_field(Field::new(
            "Test Field",
            FieldValue::Text("Test field value".into()),
        ));

        assert_eq!(entry.fields.len(), 1);
        let removed = entry
            .remove_field("Test Field")
            .expect("should remove existing field");
        assert_eq!(entry.fields.len(), 0);
        assert_eq!(removed.label, "Test Field");
    }
}
