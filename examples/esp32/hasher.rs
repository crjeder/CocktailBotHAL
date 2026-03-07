// examples/esp32/hasher.rs
//
// Password hasher for ESP32 using PBKDF2-HMAC-SHA256.
//
// Hash format: `pbkdf2$<iterations>$<salt_hex>$<dk_hex>`
//   - iterations : decimal u32 (default ITERATIONS)
//   - salt_hex   : hex-encoded 4-byte salt
//   - dk_hex     : hex-encoded 32-byte derived key
//
// Salt is generated from a monotonic counter.
// TODO (ESP32 bring-up): seed SALT_CTR from esp_hal::rng::Rng for
// cryptographic-quality randomness.

use alloc::format;
use alloc::string::String;
use alloc::vec;
use core::sync::atomic::{AtomicU32, Ordering};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use cocktail_bot_hal::hal::{ErrorInfo, PasswordHasher};

const ITERATIONS: u32 = 10_000;
const DK_LEN: usize = 32;

/// Monotonic counter used to produce a distinct salt on each `hash` call.
static SALT_CTR: AtomicU32 = AtomicU32::new(0);

/// Password hasher for ESP32 using PBKDF2-HMAC-SHA256.
pub struct Esp32PasswordHasher;

impl PasswordHasher for Esp32PasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        let salt_n = SALT_CTR.fetch_add(1, Ordering::Relaxed);
        let salt = salt_n.to_le_bytes();
        let mut dk = [0u8; DK_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, ITERATIONS, &mut dk);
        Ok(format!(
            "pbkdf2${}${}${}",
            ITERATIONS,
            bytes_to_hex(&salt),
            bytes_to_hex(&dk)
        ))
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        // Parse format: pbkdf2$<iterations>$<salt_hex>$<dk_hex>
        let mut parts = stored_hash.splitn(4, '$');
        let scheme = parts.next().unwrap_or("");
        let iters: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let salt_hex = parts.next().unwrap_or("");
        let dk_hex = parts.next().unwrap_or("");

        if scheme != "pbkdf2" || iters == 0 || salt_hex.is_empty() || dk_hex.is_empty() {
            return false;
        }

        let salt = match hex_to_bytes(salt_hex) {
            Some(s) => s,
            None => return false,
        };
        let expected_dk = match hex_to_bytes(dk_hex) {
            Some(d) => d,
            None => return false,
        };

        let mut dk = vec![0u8; expected_dk.len()];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, iters, &mut dk);
        constant_time_eq(&dk, &expected_dk)
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_to_bytes(hex: &str) -> Option<alloc::vec::Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = alloc::vec::Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
