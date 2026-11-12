// src/core/user.rs

use super::vault::Vault;

#[derive(Debug)]
pub struct User {
    pub username: String,
    vaults: Vec<Vault>,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            vaults: Vec::new(),
        }
    }

    pub fn add_vault(&mut self, vault: Vault) {
        self.vaults.push(vault);
    }

    pub fn get_vault_mut(&mut self, id: &str) -> Option<&mut Vault> {
        self.vaults.iter_mut().find(|v| v.id == id)
    }

    pub fn get_vault(&self, id: &str) -> Option<&Vault> {
        self.vaults.iter().find(|v| v.id == id)
    }

    pub fn vaults(&self) -> &[Vault] {
        &self.vaults
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_has_empty_vaults() {
        let user = User::new("test_user".to_string());
        assert_eq!(user.username, "test_user");
        assert_eq!(user.vaults().len(), 0);
    }

    #[test]
    fn test_add_and_retrieve_vault() {
        let mut user = User::new("test_user".to_string());
        let vault = Vault::new("v-123".to_string(), "Bank Vault".to_string());

        user.add_vault(vault);

        assert_eq!(user.vaults().len(), 1);

        let found = user.get_vault("v-123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Bank Vault");

        let missing = user.get_vault("non-existant");
        assert!(missing.is_none());
    }
}
