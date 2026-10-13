//! ML-DSA-87 (FIPS 204) ≈ CRYSTALS-Dilithium-5 — постквантовые подписи без OpenSSL.

use fips204::ml_dsa_87;
use fips204::traits::{SerDes, Signer, Verifier};
use rand_core::OsRng;

use crate::{DILITHIUM5_PK, DILITHIUM5_SIG, DILITHIUM5_SK};

/// Генерация ключей Dilithium-5 / ML-DSA-87.
/// (публичный 2592, секретный 4896).
pub fn dilithium5_генерация_ключей() -> Result<(Vec<u8>, Vec<u8>), String> {
    let (pk, sk) = ml_dsa_87::try_keygen_with_rng(&mut OsRng)
        .map_err(|e| format!("dilithium keygen: {e:?}"))?;
    let pk_b = pk.into_bytes().to_vec();
    let sk_b = sk.into_bytes().to_vec();
    if pk_b.len() != DILITHIUM5_PK || sk_b.len() != DILITHIUM5_SK {
        return Err(format!(
            "unexpected sizes pk={} sk={} (want {}/{})",
            pk_b.len(),
            sk_b.len(),
            DILITHIUM5_PK,
            DILITHIUM5_SK
        ));
    }
    Ok((pk_b, sk_b))
}

/// Подпись сообщения → 4627 байт (FIPS 204 ML-DSA-87).
pub fn dilithium5_подписать(сообщение: &[u8], секретный: &[u8]) -> Result<Vec<u8>, String> {
    if секретный.len() != DILITHIUM5_SK {
        return Err(format!("sk len {}, want {}", секретный.len(), DILITHIUM5_SK));
    }
    let arr: [u8; DILITHIUM5_SK] = секретный
        .try_into()
        .map_err(|_| "sk bytes".to_string())?;
    let sk = ml_dsa_87::PrivateKey::try_from_bytes(arr)
        .map_err(|e| format!("sk parse: {e:?}"))?;
    let sig = sk
        .try_sign_with_rng(&mut OsRng, сообщение, &[])
        .map_err(|e| format!("sign: {e:?}"))?;
    let out = sig.to_vec();
    if out.len() != DILITHIUM5_SIG {
        return Err(format!("sig len {}, want {}", out.len(), DILITHIUM5_SIG));
    }
    Ok(out)
}

/// Проверка подписи.
pub fn dilithium5_проверить(сообщение: &[u8], подпись: &[u8], публичный: &[u8]) -> bool {
    if публичный.len() != DILITHIUM5_PK || подпись.len() != DILITHIUM5_SIG {
        return false;
    }
    let Ok(pk_arr) = <[u8; DILITHIUM5_PK]>::try_from(публичный) else {
        return false;
    };
    let Ok(pk) = ml_dsa_87::PublicKey::try_from_bytes(pk_arr) else {
        return false;
    };
    let Ok(sig) = <[u8; DILITHIUM5_SIG]>::try_from(подпись) else {
        return false;
    };
    pk.verify(сообщение, &sig, &[])
}

#[cfg(test)]
mod тесты {
    use super::*;

    #[test]
    fn подпись_round_trip() {
        let (pk, sk) = dilithium5_генерация_ключей().unwrap();
        assert_eq!(pk.len(), DILITHIUM5_PK);
        assert_eq!(sk.len(), DILITHIUM5_SK);
        let msg = "важное сообщение".as_bytes();
        let sig = dilithium5_подписать(msg, &sk).unwrap();
        assert_eq!(sig.len(), DILITHIUM5_SIG);
        assert!(dilithium5_проверить(msg, &sig, &pk));
    }

    #[test]
    fn подделка() {
        let (pk, sk) = dilithium5_генерация_ключей().unwrap();
        let sig = dilithium5_подписать("оригинальное сообщение".as_bytes(), &sk).unwrap();
        assert!(!dilithium5_проверить(
            "подделанное сообщение".as_bytes(),
            &sig,
            &pk
        ));
    }
}
