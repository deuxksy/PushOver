use worker::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_signature(
    payload: &str,
    signature: &str,
    secret: &str,
) -> Result<bool> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| Error::from("Invalid secret key"))?;

    mac.update(payload.as_bytes());
    let expected = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected);

    // Constant-time comparison
    let sig_bytes = signature.as_bytes();
    let exp_bytes = expected_hex.as_bytes();

    let mut result: u8 = (sig_bytes.len() ^ exp_bytes.len()) as u8;

    for i in 0..sig_bytes.len().max(exp_bytes.len()) {
        let s = if i < sig_bytes.len() { sig_bytes[i] } else { 0 };
        let e = if i < exp_bytes.len() { exp_bytes[i] } else { 0 };
        result |= s ^ e;
    }

    Ok(result == 0)
}

pub fn generate_signature(payload: &str, secret: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| Error::from("Invalid secret key"))?;

    mac.update(payload.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}
