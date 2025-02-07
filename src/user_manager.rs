use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;

pub struct UserManager {
    users: HashMap<String, String>,
}

impl UserManager {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("admin".to_string(), hash("admin123", DEFAULT_COST).unwrap());
        UserManager { users }
    }

    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        if let Some(stored_password) = self.users.get(username) {
            return verify(password, stored_password).unwrap_or(false);
        }
        false
    }

    pub fn add_user(&mut self, username: String, password: String) {
        if !self.users.contains_key(&username) {
            let hashed_password = hash(&password, DEFAULT_COST).unwrap();
            self.users.insert(username.clone(), hashed_password);
            println!("User '{}' created successfully!", username);
        } else {
            println!("User '{}' already exists!", username);
        }
    }
}