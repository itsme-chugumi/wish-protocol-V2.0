use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_ephemeral_key() -> (StaticSecret, PublicKey) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret, public)
}

pub fn get_public_key(secret: &StaticSecret) -> PublicKey {
    PublicKey::from(secret)
}

pub fn derive_session_key(
    my_private: &StaticSecret,
    peer_public: &[u8; 32],
    requester_id: &str,
    responder_id: &str,
) -> Result<[u8; 32]> {
    let peer_public_key = PublicKey::from(*peer_public);
    let shared_secret = my_private.diffie_hellman(&peer_public_key);

    let hk = Hkdf::<Sha256>::new(
        Some(b"WishProtocol-v2.0-SessionKey"),
        shared_secret.as_bytes(),
    );

    let mut session_key = [0u8; 32];
    let info = format!("{}{}", requester_id, responder_id);

    hk.expand(info.as_bytes(), &mut session_key)
        .map_err(|_| anyhow!("HKDF expansion failed"))?;

    Ok(session_key)
}

fn build_nonce(counter: u32, timestamp: u32) -> [u8; 12] {
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[0..8].copy_from_slice(&(counter as u64).to_be_bytes());
    nonce_bytes[8..12].copy_from_slice(&timestamp.to_be_bytes());
    nonce_bytes
}

pub fn encrypt_message(
    session_key: &[u8; 32],
    counter: u32,
    timestamp: u32,
    message: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(session_key.into());
    let nonce_bytes = build_nonce(counter, timestamp);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: message,
        aad,
    };

    cipher
        .encrypt(nonce, payload)
        .map_err(|e| anyhow!("Encryption failed: {}", e))
}

pub fn decrypt_message(
    session_key: &[u8; 32],
    counter: u32,
    timestamp: u32,
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(session_key.into());
    let nonce_bytes = build_nonce(counter, timestamp);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: ciphertext,
        aad,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|e| anyhow!("Decryption failed: {}", e))
}

pub fn zeroize_key(key: &mut [u8; 32]) {
    key.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange_and_encryption() {
        let (alice_secret, alice_public) = generate_ephemeral_key();
        let (bob_secret, bob_public) = generate_ephemeral_key();

        let alice_session_key = derive_session_key(
            &alice_secret,
            bob_public.as_bytes(),
            "alice-12345678",
            "bob-87654321",
        ).unwrap();

        let bob_session_key = derive_session_key(
            &bob_secret,
            alice_public.as_bytes(),
            "alice-12345678",
            "bob-87654321",
        ).unwrap();

        assert_eq!(alice_session_key, bob_session_key);

        let message = b"Hello, Wish Protocol!";
        let counter = 1;
        let timestamp = 1234567890;
        let aad = b"\x02alice-12345678bob-87654321";

        let ciphertext = encrypt_message(
            &alice_session_key,
            counter,
            timestamp,
            message,
            aad,
        ).unwrap();

        let decrypted = decrypt_message(
            &bob_session_key,
            counter,
            timestamp,
            &ciphertext,
            aad,
        ).unwrap();

        assert_eq!(message.to_vec(), decrypted);
    }

    #[test]
    fn test_decrypt_failure_with_wrong_key() {
        let (alice_secret, alice_public) = generate_ephemeral_key();
        let (bob_secret, _bob_public) = generate_ephemeral_key();
        let (_charlie_secret, charlie_public) = generate_ephemeral_key();

        let alice_session_key = derive_session_key(
            &alice_secret,
            charlie_public.as_bytes(),
            "alice",
            "charlie",
        ).unwrap();

        let bob_session_key = derive_session_key(
            &bob_secret,
            alice_public.as_bytes(),
            "alice",
            "bob",
        ).unwrap();

        let message = b"Secret";
        let counter = 1;
        let timestamp = 100;
        let aad = b"\x02";

        let ciphertext = encrypt_message(&alice_session_key, counter, timestamp, message, aad).unwrap();
        let result = decrypt_message(&bob_session_key, counter, timestamp, &ciphertext, aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_aad_mismatch_fails() {
        let (alice_secret, _alice_public) = generate_ephemeral_key();
        let (_bob_secret, bob_public) = generate_ephemeral_key();

        let session_key = derive_session_key(
            &alice_secret,
            bob_public.as_bytes(),
            "alice",
            "bob",
        ).unwrap();

        let message = b"Secret";
        let counter = 1;
        let timestamp = 100;

        let ciphertext = encrypt_message(&session_key, counter, timestamp, message, b"correct_aad").unwrap();
        let result = decrypt_message(&session_key, counter, timestamp, &ciphertext, b"wrong_aad");

        assert!(result.is_err());
    }
}
