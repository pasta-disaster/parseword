// src/cli/dto.rs

use serde::Serialize;

use crate::core::field::{Field, FieldValue};

#[derive(Serialize)]
pub struct VaultInspectDto<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub is_locked: bool,
    pub entry_count: usize,
}

#[derive(Serialize)]
pub struct EntrySummaryDto<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub field_count: usize,
}

#[derive(Serialize)]
pub struct EntryCreatedDto<'a> {
    pub id: &'a str,
    pub title: &'a str,
}

#[derive(Serialize)]
pub struct EntryDetailDto<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub fields: Vec<FieldDto<'a>>,
}

#[derive(Serialize)]
pub struct FieldDto<'a> {
    pub label: &'a str,
    #[serde(rename = "type")]
    pub field_type: &'a str,
    pub value: String,
}

impl<'a> From<&'a Field> for FieldDto<'a> {
    fn from(field: &'a Field) -> Self {
        match &field.value {
            FieldValue::Text(v) => FieldDto {
                label: &field.label,
                field_type: "text",
                value: v.clone(),
            },
            FieldValue::Password(v) => FieldDto {
                label: &field.label,
                field_type: "password",
                value: v.clone(),
            },
            FieldValue::Note(v) => FieldDto {
                label: &field.label,
                field_type: "note",
                value: v.clone(),
            },
            FieldValue::File { filename, bytes } => FieldDto {
                label: &field.label,
                field_type: "file",
                value: format!("{} ({} bytes)", filename, bytes.len()),
            },
        }
    }
}

#[derive(Serialize)]
pub struct VaultInitDto<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub path: String,
}

#[derive(Serialize)]
pub struct OperationSuccessDto<'a> {
    pub status: &'a str,
    pub message: String,
}
