mod core;

use crate::core::user::User;
use crate::core::vault::Vault;

fn main() {
    println!("Initialising Parseword backend");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_model_hierarchy() {
        // create user
        let mut user = User::new("cypher".to_string());
        assert_eq!(user.username, "cypher");
        assert_eq!(user.vaults().len(), 0);

        // create vault
        let vault = Vault::new("vault-001".to_string(), "Personal Vault".to_string());
        user.add_vault(vault);

        assert_eq!(user.vaults().len(), 1);

        let retrieved = user.get_vault("vault-001");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Personal Vault");

        // test locking/unlocking
        let vault_mut = user.get_vault_mut("vault-001").unwrap();
        assert!(vault_mut.is_locked);

        vault_mut.unlock(Vec::new());
        assert!(!vault_mut.is_locked);
        assert!(vault_mut.entries().is_some());

        vault_mut.lock();
        assert!(vault_mut.is_locked);
        assert!(vault_mut.entries().is_none());
    }
}
