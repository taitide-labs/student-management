use crate::{hash_password, verify_password};

#[test]
fn password_hash_verifies_matching_secret_and_rejects_mismatch() {
    let hash = hash_password("secret-123").expect("hash password");
    assert!(verify_password("secret-123", &hash));
    assert!(!verify_password("wrong", &hash));
}
