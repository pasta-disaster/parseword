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
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
