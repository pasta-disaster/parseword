use super::entry::Entry;

#[derive(Debug)]
pub struct Vault {
    pub id: String,
    pub name: String,
    pub is_locked: bool,
    entries: Option<Vec<Entry>>,
}

impl Vault {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_starts_locked() {
        let vault = Vault::new("v1".to_string(), "Test Vault".to_string());
        assert!(vault.is_locked);
        assert!(vault.entries().is_none());
    }

    #[test]
    fn test_vault_unlock_and_lock() {
        let mut vault = Vault::new("v1".to_string(), "Test Vault".to_string());

        // unlock
        vault.unlock(Vec::new());
        assert!(!vault.is_locked);
        assert!(vault.entries().is_some());

        // lock
        vault.lock();
        assert!(vault.is_locked);
        assert!(vault.entries.is_none());
    }
}
