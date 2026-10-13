//! Криптография ZIP **без OpenSSL**.
//!
//! - SHA-256 / SHA-3-256 (RustCrypto)
//! - AES-256-GCM (RustCrypto)
//! - ML-KEM-1024 ≈ Kyber-1024 (FIPS 203)
//! - ML-DSA-87 ≈ Dilithium-5 (FIPS 204)
//! - getrandom (ОС CSPRNG)
//! - constant-time сравнение

pub mod aes_gcm;
pub mod kem;
pub mod sha;
pub mod sign;
pub mod util;

pub use aes_gcm::{
    аес256_гцм_зашифровать, аес256_гцм_зашифровать_с_iv, аес256_гцм_расшифровать, AesGcmPacket,
};
pub use kem::{kyber1024_генерация_ключей, kyber1024_декапсуляция, kyber1024_инкапсуляция};
pub use sha::{ша256, ша3_256};
pub use sign::{dilithium5_генерация_ключей, dilithium5_подписать, dilithium5_проверить};
pub use util::{constant_time_eq, случайные_байты};

/// Размеры ML-KEM-1024 / Kyber-1024 (байты).
pub const KYBER1024_PK: usize = 1568;
pub const KYBER1024_SK: usize = 3168;
pub const KYBER1024_CT: usize = 1568;
pub const KYBER1024_SS: usize = 32;

/// Размеры ML-DSA-87 / Dilithium-5 (FIPS 204).
pub const DILITHIUM5_PK: usize = 2592;
pub const DILITHIUM5_SK: usize = 4896;
pub const DILITHIUM5_SIG: usize = 4627;
