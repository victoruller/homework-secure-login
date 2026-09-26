use argon2::{
    Argon2, 
    password_hash::{
        PasswordHasher, 
        PasswordVerifier
    },
};

pub fn generate_hash(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .unwrap()
        .to_string()
}


pub fn verify_password(password: &str, hash: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok()
}