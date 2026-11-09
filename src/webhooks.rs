use crate::errors::WebhookError;
use ring::hmac;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Webhook signature validator for Aeitron events.
pub struct WebhookVerifier;

impl WebhookVerifier {
    /// Cryptographically validates the `Aeitron-Signature` header against the raw payload bytes.
    ///
    /// Header specification: `t=1730000000,v1=9f86d081884c...`
    /// Signed content: `${timestamp}.${payload}`
    pub fn verify(
        payload: &[u8],
        signature_header: &str,
        secret: &str,
        tolerance: Duration,
    ) -> Result<(), WebhookError> {
        let secret = secret.trim();
        if secret.is_empty() {
            return Err(WebhookError::MissingSecret);
        }

        let mut timestamp_str = None;
        let mut signatures = Vec::new();

        for part in signature_header.split(',') {
            let mut kv = part.trim().splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                match k {
                    "t" => timestamp_str = Some(v),
                    "v1" => signatures.push(v),
                    _ => {}
                }
            }
        }

        let timestamp_str = timestamp_str
            .ok_or_else(|| WebhookError::InvalidHeader(signature_header.to_string()))?;

        if signatures.is_empty() {
            return Err(WebhookError::InvalidHeader(signature_header.to_string()));
        }

        let timestamp_unix: u64 = timestamp_str
            .parse()
            .map_err(|_| WebhookError::InvalidHeader("invalid timestamp integer".to_string()))?;

        // Anti-replay attack check
        let event_time = UNIX_EPOCH + Duration::from_secs(timestamp_unix);
        let now = SystemTime::now();

        let diff = if now >= event_time {
            now.duration_since(event_time).unwrap_or(Duration::ZERO)
        } else {
            event_time.duration_since(now).unwrap_or(Duration::ZERO)
        };

        if tolerance > Duration::ZERO && diff > tolerance {
            return Err(WebhookError::TimestampExpired {
                age: diff,
                tolerance,
            });
        }

        // Construct signed payload: "${timestamp}.${payload}"
        let mut signed_data = Vec::with_capacity(timestamp_str.len() + 1 + payload.len());
        signed_data.extend_from_slice(timestamp_str.as_bytes());
        signed_data.push(b'.');
        signed_data.extend_from_slice(payload);

        let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());

        // Check each provided signature using constant-time comparison
        for sig_hex in signatures {
            if let Ok(sig_bytes) = decode_hex(sig_hex) {
                if hmac::verify(&key, &signed_data, &sig_bytes).is_ok() {
                    return Ok(());
                }
            }
        }

        Err(WebhookError::SignatureMismatch)
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ()> {
    if !s.len().is_multiple_of(2) {
        return Err(());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
        .collect()
}
