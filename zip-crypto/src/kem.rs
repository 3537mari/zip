//! ML-KEM-1024 (FIPS 203) ≈ CRYSTALS-Kyber-1024 — постквантовый KEM без OpenSSL.

use ml_kem::kem::{Decapsulate, Encapsulate};
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem1024};
use rand_core::OsRng;

use crate::{KYBER1024_CT, KYBER1024_PK, KYBER1024_SK, KYBER1024_SS};

type Dk = <MlKem1024 as KemCore>::DecapsulationKey;
type Ek = <MlKem1024 as KemCore>::EncapsulationKey;

/// Генерация пары ключей Kyber-1024 / ML-KEM-1024.
/// Возвращает (публичный 1568, секретный 3168).
pub fn kyber1024_генерация_ключей() -> Result<(Vec<u8>, Vec<u8>), String> {
    let (dk, ek) = MlKem1024::generate(&mut OsRng);
    let pk = ek.as_bytes().to_vec();
    let sk = dk.as_bytes().to_vec();
    if pk.len() != KYBER1024_PK || sk.len() != KYBER1024_SK {
        return Err(format!(
            "unexpected key sizes pk={} sk={} (want {}/{})",
            pk.len(),
            sk.len(),
            KYBER1024_PK,
            KYBER1024_SK
        ));
    }
    Ok((pk, sk))
}

/// Инкапсуляция: (ciphertext 1568, shared_secret 32).
pub fn kyber1024_инкапсуляция(публичный: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    if публичный.len() != KYBER1024_PK {
        return Err(format!("pk len {}, want {}", публичный.len(), KYBER1024_PK));
    }
    let enc: ml_kem::Encoded<Ek> = публичный
        .try_into()
        .map_err(|_| "pk encode".to_string())?;
    let ek = Ek::from_bytes(&enc);
    let (ct, ss) = ek
        .encapsulate(&mut OsRng)
        .map_err(|_| "encapsulate failed".to_string())?;
    let ct_bytes = ct.to_vec();
    let ss_bytes = ss.to_vec();
    if ct_bytes.len() != KYBER1024_CT || ss_bytes.len() != KYBER1024_SS {
        return Err("unexpected encapsulate sizes".into());
    }
    Ok((ct_bytes, ss_bytes))
}

/// Декапсуляция → shared_secret 32.
pub fn kyber1024_декапсуляция(шифротекст: &[u8], секретный: &[u8]) -> Result<Vec<u8>, String> {
    if шифротекст.len() != KYBER1024_CT {
        return Err(format!("ct len {}, want {}", шифротекст.len(), KYBER1024_CT));
    }
    if секретный.len() != KYBER1024_SK {
        return Err(format!("sk len {}, want {}", секретный.len(), KYBER1024_SK));
    }
    let sk_enc: ml_kem::Encoded<Dk> = секретный
        .try_into()
        .map_err(|_| "sk encode".to_string())?;
    let dk = Dk::from_bytes(&sk_enc);
    let ct: Ciphertext<MlKem1024> = шифротекст
        .try_into()
        .map_err(|_| "ct encode".to_string())?;
    let ss = dk
        .decapsulate(&ct)
        .map_err(|_| "decapsulate failed".to_string())?;
    Ok(ss.to_vec())
}

#[cfg(test)]
mod тесты {
    use super::*;
    use crate::util::constant_time_eq;

    #[test]
    fn kem_round_trip() {
        let (pk, sk) = kyber1024_генерация_ключей().unwrap();
        assert_eq!(pk.len(), KYBER1024_PK);
        assert_eq!(sk.len(), KYBER1024_SK);
        let (ct, k_enc) = kyber1024_инкапсуляция(&pk).unwrap();
        assert_eq!(ct.len(), KYBER1024_CT);
        assert_eq!(k_enc.len(), KYBER1024_SS);
        let k_dec = kyber1024_декапсуляция(&ct, &sk).unwrap();
        assert!(constant_time_eq(&k_enc, &k_dec), "K_enc != K_dec");
    }
}
