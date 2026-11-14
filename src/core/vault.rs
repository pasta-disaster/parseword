use super::entry::Entry;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Vault {
    pub id: String,
    pub name: String,
    pub is_locked: bool,
    entries: Option<Vec<Entry>>,
}

impl Vault {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_locked: true,
            entries: None,
        }
    }

    pub fn unlock(&mut self, decrypted_entries: Vec<Entry>) {
        self.entries = Some(decrypted_entries);
        self.is_locked = false;
    }

    pub fn lock(&mut self) {
        self.entries = None;
        self.is_locked = true;
    }

    pub fn entries(&self) -> Option<&[Entry]> {
        self.entries.as_deref()
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), &'static str> {
        match &mut self.entries {
            Some(entries) => {
                entries.push(entry);
                Ok(())
            }
            None => Err("Cannot add entry to a locked vault"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::field::{Field, FieldValue};

    #[test]
    fn test_vault_starts_locked() {
        let vault = Vault::new("v1".to_string(), "Test Vault");
        assert!(vault.is_locked);
        assert!(vault.entries().is_none());
    }

    #[test]
    fn test_vault_unlock_and_lock() {
        let mut vault = Vault::new("v1".to_string(), "Test Vault");

        // unlock
        vault.unlock(Vec::new());
        assert!(!vault.is_locked);
        assert!(vault.entries().is_some());

        // lock
        vault.lock();
        assert!(vault.is_locked);
        assert!(vault.entries.is_none());
    }

    #[test]
    fn test_add_entry_behaviour() {
        let mut vault = Vault::new("vault-001".to_string(), "Test vault".to_string());
        let entry = Entry::new(
            "entry-001",
            "Test Service",
            vec![Field::new(
                "Username",
                FieldValue::Text("test-username".to_string()),
            )],
        );

        // adding entry to locked vault should fail
        assert!(vault.add_entry(entry.clone()).is_err());

        vault.unlock(Vec::new());
        assert!(vault.add_entry(entry).is_ok());
        assert_eq!(vault.entries().unwrap().len(), 1);
    }
}
