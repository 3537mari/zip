//! SHA-256 и SHA-3-256 без OpenSSL.

use sha2::{Digest, Sha256};
use sha3::Sha3_256;

pub fn ша256(данные: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(данные);
    h.finalize().into()
}

pub fn ша256_hex(данные: &[u8]) -> String {
    hex::encode(ша256(данные))
}

pub fn ша3_256(данные: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(данные);
    h.finalize().into()
}

#[cfg(test)]
mod тесты {
    use super::*;

    #[test]
    fn nist_empty() {
        assert_eq!(
            ша256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn nist_abc() {
        assert_eq!(
            ша256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn ru_привет_мир() {
        assert_eq!(
            ша256_hex("привет мир".as_bytes()),
            "527c01ee137e87eaba739dc590ea562e9114057d3f7aba84ff429ba8ccf3b6ad"
        );
    }
}
