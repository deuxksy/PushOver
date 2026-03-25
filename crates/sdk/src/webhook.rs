use crate::{PushOverError, WebhookPayload};

pub fn verify_webhook_signature(
    signature: &str,
    body: &str,
    secret: &str,
) -> Result<(), PushOverError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| PushOverError::InvalidSignature)?;
    mac.update(body.as_bytes());

    let expected = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected);

    if !timing_safe_equal(signature, &expected_hex) {
        return Err(PushOverError::InvalidSignature);
    }

    Ok(())
}

fn timing_safe_equal(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    if a_bytes.len() != b_bytes.len() {
        // Still compare up to shorter length to avoid timing leak
        let min_len = a_bytes.len().min(b_bytes.len());
        let mut result = 0u8;
        for i in 0..min_len {
            result |= a_bytes[i] ^ b_bytes[i];
        }
        // Account for length difference without early return
        result |= if a_bytes.len() > b_bytes.len() {
            255
        } else {
            0
        };
        return result == 0;
    }

    let mut result = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn parse_webhook_payload(body: &str) -> Result<WebhookPayload, PushOverError> {
    serde_json::from_str(body).map_err(PushOverError::SerializationError)
}
