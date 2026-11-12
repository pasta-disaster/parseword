// src/core/entry.rs

use super::field::Field;

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Entry {
    pub fn new(id: String, title: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            title,
            fields: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
