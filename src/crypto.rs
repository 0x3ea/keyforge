use argon2::{Algorithm, Argon2, Params, Version};

use crate::sensitive::SecretVec;

const KDF_DOMAIN: &str = "keyforge-argon2-v1";

pub fn build_salt(site: &str, username: &str) -> Vec<u8> {
    format!(
        "{}|{}|{}|{}|{}",
        KDF_DOMAIN,
        site.len(),
        site,
        username.len(),
        username
    )
    .into_bytes()
}

pub fn generate_key(password: &SecretVec, salt: &[u8]) -> Result<SecretVec, String> {
    let params = Params::new(65536, 3, 4, Some(64)).map_err(|e| e.to_string())?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = vec![0u8; 64];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| e.to_string())?;

    SecretVec::new(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_stable_salt() {
        assert_eq!(
            build_salt("github.com", "alice"),
            b"keyforge-argon2-v1|10|github.com|5|alice".to_vec()
        );
    }
}
