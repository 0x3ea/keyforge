use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::sensitive::SecretVec;

type HmacSha256 = Hmac<Sha256>;

const DOMAIN: &[u8] = b"keyforge-password-encode-v2";
const ALPHANUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const SYMBOLS: &[u8] = b"!@#$%^&*";
pub fn expand(seed: &[u8], length: u32, symbols: bool, counter: u64) -> Result<[u8; 32], String> {
    let mut mac = HmacSha256::new_from_slice(seed).map_err(|e| e.to_string())?;

    mac.update(DOMAIN);
    mac.update(&length.to_be_bytes());
    mac.update(&[symbols as u8]);
    mac.update(&counter.to_be_bytes());

    let bytes = mac.finalize().into_bytes();

    Ok(bytes.into())
}

pub fn encode(seed: &SecretVec, length: u32, symbols: bool) -> Result<SecretVec, String> {
    let mut charset = ALPHANUM.to_vec();

    if symbols {
        charset.extend_from_slice(SYMBOLS);
    }

    let charset_len = charset.len();
    let max_accept = 256 - (256 % charset_len);

    let mut password = Vec::with_capacity(length as usize);
    let mut counter = 0u64;

    while password.len() < length as usize {
        let block = expand(seed.as_bytes(), length, symbols, counter)?;
        counter += 1;

        for byte in block {
            if byte < max_accept as u8 {
                let index = (byte as usize) % charset_len;
                password.push(charset[index]);
            }
            if password.len() == length as usize {
                break;
            }
        }
    }
    SecretVec::new(password)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encode_is_deterministic() {
        let seed = SecretVec::new(vec![42u8; 64]).unwrap();

        let first = encode(&seed, 16, false).unwrap();
        let second = encode(&seed, 16, false).unwrap();

        assert_eq!(first.as_bytes(), second.as_bytes());
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn encode_respects_length() {
        let seed = SecretVec::new(vec![42u8; 64]).unwrap();

        assert_eq!(encode(&seed, 8, false).unwrap().len(), 8);
        assert_eq!(encode(&seed, 36, false).unwrap().len(), 36);
    }

    #[test]
    fn shorter_password_is_not_prefix_of_longer_password() {
        let seed = SecretVec::new(vec![42u8; 64]).unwrap();

        let short = encode(&seed, 8, false).unwrap();
        let long = encode(&seed, 16, false).unwrap();

        assert_ne!(short.as_bytes(), &long.as_bytes()[..short.len()]);
    }

    #[test]
    fn symbols_changes_output() {
        let seed = SecretVec::new(vec![42u8; 64]).unwrap();

        let with_symbols = encode(&seed, 16, true).unwrap();
        let without_symbols = encode(&seed, 16, false).unwrap();

        assert_ne!(with_symbols.as_bytes(), without_symbols.as_bytes());
    }

    /// Chi-squared goodness-of-fit: output characters should be uniformly
    /// distributed across the (62-char) alphanumeric set.
    ///
    /// `encode` is deterministic in the seed, so chi2 is a fixed value on every
    /// run — this is not a flaky statistical test. Rejection sampling yields a
    /// uniform distribution; a regression that drops it (e.g. mapping `byte %
    /// charset_len` directly) produces modulo bias that pushes chi2 into the
    /// hundreds, well above the threshold.
    #[test]
    fn charset_is_uniform_chi_squared() {
        let charset = ALPHANUM;
        let categories = charset.len();
        let length = 200u32;
        let seeds = 256u32;

        let mut counts = vec![0u64; categories];
        for i in 0..seeds {
            let seed = SecretVec::new(vec![i as u8; 64]).unwrap();
            let password = encode(&seed, length, false).unwrap();

            for &byte in password.as_bytes() {
                let index = charset
                    .iter()
                    .position(|&c| c == byte)
                    .expect("encoded byte must be in the charset");
                counts[index] += 1;
            }
        }

        let total: u64 = counts.iter().sum();
        let expected = total as f64 / categories as f64;
        let chi2: f64 = counts
            .iter()
            .map(|&observed| {
                let diff = observed as f64 - expected;
                diff * diff / expected
            })
            .sum();

        // df = categories - 1 = 61. Threshold = 2 * df: for a uniform source
        // P(chi2 > 2*df) is negligible (<1e-6), so this never false-fails yet
        // still flags real bias.
        let threshold = 2.0 * (categories as f64 - 1.0);
        assert!(
            chi2 < threshold,
            "chi2 = {chi2:.2} exceeds threshold {threshold:.0}; charset is not uniform"
        );
    }
}
