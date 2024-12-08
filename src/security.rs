// Path: src/security.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Password hashing error: {0}")]
    HashingError(String),
    
    #[error("Password verification error: {0}")]
    VerificationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

pub struct SecurityManager {
    email_regex: Regex,
    password_regex: Regex,
    role_regex: Regex,
    name_regex: Regex,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
            password_regex: Regex::new(r"^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$").unwrap(),
            role_regex: Regex::new(r"^(admin|user|guest)$").unwrap(),
            name_regex: Regex::new(r"^[a-zA-Z\s]{1,50}$").unwrap(),
        }
    }

    pub fn hash_password(&self, password: &str) -> SecurityResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| SecurityError::HashingError(e.to_string()))
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> SecurityResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn is_valid_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn is_valid_password(&self, password: &str) -> bool {
        self.password_regex.is_match(password)
    }

    pub fn is_valid_role(&self, role: &str) -> bool {
        self.role_regex.is_match(role)
    }

    pub fn is_valid_name(&self, name: &str) -> bool {
        self.name_regex.is_match(name)
    }

    pub fn validate_user_input(&self, email: &str, password: &str) -> SecurityResult<()> {
        if !self.is_valid_email(email) {
            return Err(SecurityError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        if !self.is_valid_password(password) {
            return Err(SecurityError::ValidationError(
                "Password must be at least 8 characters long and contain both letters and numbers"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let security = SecurityManager::new();
        let password = "TestPass123";

        let hash = security.hash_password(password).unwrap();
        assert!(security.verify_password(password, &hash).unwrap());
        assert!(!security.verify_password("WrongPass123", &hash).unwrap());
    }

    #[test]
    fn test_email_validation() {
        let security = SecurityManager::new();
        
        assert!(security.is_valid_email("test@example.com"));
        assert!(security.is_valid_email("user.name+tag@example.co.uk"));
        assert!(!security.is_valid_email("invalid.email@"));
        assert!(!security.is_valid_email("@example.com"));
    }

    #[test]
    fn test_password_validation() {
        let security = SecurityManager::new();
        
        assert!(security.is_valid_password("Password123"));
        assert!(security.is_valid_password("SecurePass1"));
        assert!(!security.is_valid_password("weak"));
        assert!(!security.is_valid_password("onlyletters"));
        assert!(!security.is_valid_password("12345678"));
    }

    #[test]
    fn test_role_validation() {
        let security = SecurityManager::new();
        
        assert!(security.is_valid_role("admin"));
        assert!(security.is_valid_role("user"));
        assert!(security.is_valid_role("guest"));
        assert!(!security.is_valid_role("superuser"));
    }

    #[test]
    fn test_name_validation() {
        let security = SecurityManager::new();
        
        assert!(security.is_valid_name("John Doe"));
        assert!(security.is_valid_name("Jane"));
        assert!(!security.is_valid_name("")); // Empty name
        assert!(!security.is_valid_name("A very very very very very very very very very very very very very very very very very long name"));
        assert!(!security.is_valid_name("Invalid_Name123"));
    }
}
