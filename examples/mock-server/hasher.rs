// examples/mock-server/hasher.rs
//
// Development-only password hasher for the mock server.
// Uses the same stub$<plaintext> scheme as examples/dev to avoid pulling
// in PBKDF2.  Do NOT use in production.

use cocktail_bot_hal::hal::{ErrorInfo, PasswordHasher};

/// Development stub — stores and verifies passwords as `stub$<plaintext>`.
pub struct MockPasswordHasher;

impl PasswordHasher for MockPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        Ok(format!("stub${}", password))
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        let expected = format!("stub${}", password);
        let a = expected.as_bytes();
        let b = stored_hash.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        let mut diff: u8 = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }
}
