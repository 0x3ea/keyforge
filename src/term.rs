use std::io;

use rpassword::prompt_password;

use crate::sensitive::SecretVec;

pub fn get_master_password(confirm: bool) -> Result<SecretVec, String> {
    loop {
        let first = prompt_password("Enter master password: ")
            .map_err(|e| format!("failed to read password: {e}"))?;

        validate_password(&first)?;

        if !confirm {
            return SecretVec::new(first.into_bytes());
        }

        let second = prompt_password("Confirm master password: ")
            .map_err(|e| format!("failed to read password: {e}"))?;

        if first != second {
            eprintln!("Passwords do not match, try again.");
            continue;
        }
        return SecretVec::new(first.into_bytes());
    }
}

pub fn confirm(prompt: &str) -> Result<bool, String> {
    eprint!("{prompt}");

    let mut input = String::new();
    let n = io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read confirmation: {e}"))?;

    if n == 0 {
        return Ok(false);
    }
    let ans = input.trim();
    Ok(ans.starts_with('y') || ans.starts_with('Y'))
}

fn validate_password(password: &str) -> Result<(), String> {
    let len = password.chars().count();

    if len < 12 {
        return Err("master password must be at least 12 characters".to_string());
    }

    if len > 128 {
        return Err("master password must be at most 128 characters".to_string());
    }

    Ok(())
}
