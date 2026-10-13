//! AES-256-GCM без OpenSSL (RustCrypto `aes-gcm`).

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use crate::util::случайные_байты;

/// Пакет: ciphertext || tag (16) с отдельным IV (12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AesGcmPacket {
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16],
    pub iv: [u8; 12],
}

/// Зашифровать с заданным IV (для NIST-векторов).
pub fn аес256_гцм_зашифровать(
    plaintext: &[u8],
    key: &[u8; 32],
    iv: &[u8; 12],
) -> Result<(Vec<u8>, [u8; 16]), String> {
    аес256_гцм_зашифровать_aad(plaintext, key, iv, &[])
}

pub fn аес256_гцм_зашифровать_aad(
    plaintext: &[u8],
    key: &[u8; 32],
    iv: &[u8; 12],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; 16]), String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(iv);
    let out = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| "AES-GCM encrypt failed".to_string())?;
    if out.len() < 16 {
        return Err("AES-GCM: short output".into());
    }
    let (ct, tag_bytes) = out.split_at(out.len() - 16);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(tag_bytes);
    Ok((ct.to_vec(), tag))
}

/// Зашифровать со случайным IV.
pub fn аес256_гцм_зашифровать_с_iv(
    plaintext: &[u8],
    key: &[u8; 32],
) -> Result<AesGcmPacket, String> {
    let iv_bytes = случайные_байты(12)?;
    let mut iv = [0u8; 12];
    iv.copy_from_slice(&iv_bytes);
    let (ciphertext, tag) = аес256_гцм_зашифровать(plaintext, key, &iv)?;
    Ok(AesGcmPacket {
        ciphertext,
        tag,
        iv,
    })
}

pub fn аес256_гцм_расшифровать(
    ciphertext: &[u8],
    key: &[u8; 32],
    iv: &[u8; 12],
    tag: &[u8; 16],
) -> Result<Vec<u8>, String> {
    аес256_гцм_расшифровать_aad(ciphertext, key, iv, tag, &[])
}

pub fn аес256_гцм_расшифровать_aad(
    ciphertext: &[u8],
    key: &[u8; 32],
    iv: &[u8; 12],
    tag: &[u8; 16],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(iv);
    let mut combined = Vec::with_capacity(ciphertext.len() + 16);
    combined.extend_from_slice(ciphertext);
    combined.extend_from_slice(tag);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: &combined,
                aad,
            },
        )
        .map_err(|_| "AES-GCM: authentication failed".to_string())
}

#[cfg(test)]
mod тесты {
    use super::*;
    use hex_literal::hex;

    /// NIST / CAVS-style: all-zero key, IV, 16-byte PT.
    #[test]
    fn nist_vector_1() {
        let key = [0u8; 32];
        let iv = [0u8; 12];
        let plaintext = [0u8; 16];
        let (ct, tag) = аес256_гцм_зашифровать(&plaintext, &key, &iv).unwrap();
        assert_eq!(hex::encode(&ct), "cea7403d4d606b6e074ec5d3baf39d18");
        assert_eq!(hex::encode(tag), "d0d1c8a799996bf0265b98b5d48ab919");
    }

    /// NIST SP 800-38D Test Case (McGrew/Viega style) — 60-byte PT.
    #[test]
    fn nist_vector_2() {
        let key = hex!("feffe9928665731c6d6a8f9467308308feffe9928665731c6d6a8f9467308308");
        let iv = hex!("cafebabefacedbaddecaf888");
        let plaintext = hex!(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39"
        );
        let aad = hex!("feedfacedeadbeeffeedfacedeadbeefabaddad2");
        let (ct, tag) = аес256_гцм_зашифровать_aad(&plaintext, &key, &iv, &aad).unwrap();
        assert_eq!(
            hex::encode(&ct),
            "522dc1f099567d07f47f37a32a84427d643a8cdcbfe5c0c97598a2bd2555d1aa8cb08e48590dbb3da7b08b1056828838c5f61e6393ba7a0abcc9f662"
        );
        assert_eq!(hex::encode(tag), "76fc6ece0f4e1768cddf8853bb2d551b");
    }

    #[test]
    fn round_trip() {
        let mut key = [0u8; 32];
        key.copy_from_slice(&случайные_байты(32).unwrap());
        let msg = "секретное сообщение".as_bytes();
        let pkt = аес256_гцм_зашифровать_с_iv(msg, &key).unwrap();
        let plain =
            аес256_гцм_расшифровать(&pkt.ciphertext, &key, &pkt.iv, &pkt.tag).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn подделка_тега() {
        let mut key = [0u8; 32];
        key.copy_from_slice(&случайные_байты(32).unwrap());
        let pkt = аес256_гцм_зашифровать_с_iv("секрет".as_bytes(), &key).unwrap();
        let mut bad = pkt.tag;
        bad[0] ^= 1;
        assert!(аес256_гцм_расшифровать(&pkt.ciphertext, &key, &pkt.iv, &bad).is_err());
    }
}
