// src/core/field.rs

#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Password(String),
    File { filename: String, bytes: Vec<u8> },
    Note(String),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub value: FieldValue,
}
