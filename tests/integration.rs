use keyforge::{
    crypto::{build_salt, generate_key},
    encode::encode,
    sensitive::SecretVec,
};

/// Helper: run the full derivation pipeline and return the password as bytes.
fn derive_password(
    master: &[u8],
    site: &str,
    username: &str,
    length: u32,
    symbols: bool,
) -> Vec<u8> {
    let password = SecretVec::new(master.to_vec()).unwrap();
    let salt = build_salt(site, username);
    let key = generate_key(&password, &salt).unwrap();
    let generated = encode(&key, length, symbols).unwrap();
    generated.as_bytes().to_vec()
}

#[test]
fn same_input_produces_same_output() {
    let a = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "alice",
        16,
        false,
    );
    let b = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "alice",
        16,
        false,
    );
    assert_eq!(a, b);
}

#[test]
fn different_master_password_produces_different_output() {
    let a = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "alice",
        16,
        false,
    );
    let b = derive_password(
        b"another-master-password!",
        "github.com",
        "alice",
        16,
        false,
    );
    assert_ne!(a, b);
}

#[test]
fn different_site_produces_different_output() {
    let a = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "alice",
        16,
        false,
    );
    let b = derive_password(
        b"correct-horse-battery-staple",
        "gitlab.com",
        "alice",
        16,
        false,
    );
    assert_ne!(a, b);
}

#[test]
fn different_username_produces_different_output() {
    let a = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "alice",
        16,
        false,
    );
    let b = derive_password(
        b"correct-horse-battery-staple",
        "github.com",
        "bob",
        16,
        false,
    );
    assert_ne!(a, b);
}
