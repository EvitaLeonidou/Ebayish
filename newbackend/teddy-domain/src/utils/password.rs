use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, Secret};

pub fn hash_password(password: Secret<String>) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}