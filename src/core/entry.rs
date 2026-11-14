// src/core/entry.rs

use super::field::Field;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
    pub created_at: u64,
    pub updated_at: u64,
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
    fn test_entry_field_management() {
        let mut entry = Entry::new(
            "test-service-id",
            "Test Service",
            vec![Field::new(
                "Username",
                FieldValue::Text("test-username".into()),
            )],
        );
        assert_eq!(entry.fields.len(), 1);

        entry.add_field(Field::new(
            "Password",
            FieldValue::Password("test-password".into()),
        ));
        assert_eq!(entry.fields.len(), 2);

        assert!(entry.find_field("Password").is_some());
    }
}
