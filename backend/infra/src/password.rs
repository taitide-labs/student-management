use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::error::InfraError;

#[must_use]
pub fn generate_random_secret() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn hash_password(password: &str) -> Result<String, InfraError> {
    let hash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|err| InfraError::PasswordHash(err.to_string()))?;
    Ok(hash.to_string())
}

#[must_use]
pub fn verify_password(password: &str, encoded_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
