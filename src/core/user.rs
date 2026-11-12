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



