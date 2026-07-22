use std::sync::Once;

use zeroize::Zeroize;
pub struct SecretVec {
    data: Vec<u8>,
    locked: bool,
}

static MLOCK_WARN: Once = Once::new();

fn warn_memory_lock_failed(err: &std::io::Error) {
    MLOCK_WARN.call_once(|| {
        eprintln!(
            "warning: failed to lock memory (mlock: {err}); \
                     secrets will not be pinned in RAM. Continuing anyway."
        );
    });
}

impl SecretVec {
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        let mut secret = Self {
            data,
            locked: false,
        };

        #[cfg(unix)]
        {
            if !secret.data.is_empty() {
                let result = unsafe {
                    libc::mlock(
                        secret.data.as_ptr() as *const libc::c_void,
                        secret.data.len(),
                    )
                };

                if result != 0 {
                    let err = std::io::Error::last_os_error();
                    warn_memory_lock_failed(&err);
                } else {
                    secret.locked = true;
                }
            }
        }

        Ok(secret)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecretVec {
    fn drop(&mut self) {
        self.data.zeroize();

        #[cfg(unix)]
        {
            if self.locked && !self.data.is_empty() {
                unsafe {
                    libc::munlock(self.data.as_ptr() as *const libc::c_void, self.data.len())
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_bytes_and_accessors() {
        let secret = SecretVec::new(vec![1, 2, 3, 4, 5]).unwrap();

        assert_eq!(secret.as_bytes(), &[1, 2, 3, 4, 5]);
        assert_eq!(secret.len(), 5);
        assert!(!secret.is_empty());
    }

    #[test]
    fn empty_secret_reports_empty() {
        let secret = SecretVec::new(Vec::new()).unwrap();

        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
        assert!(secret.as_bytes().is_empty());
    }

    #[test]
    fn drop_does_not_panic() {
        // Exercises the Drop path (zeroize + munlock when locked). mlock success is
        // environment-dependent (CI often has RLIMIT_MEMLOCK=0), so this is a smoke
        // test only — we cannot observe zeroization from the outside.
        for len in [0usize, 1, 64, 4096] {
            let secret = SecretVec::new(vec![0xAB; len]).unwrap();
            drop(secret);
        }
    }
}
