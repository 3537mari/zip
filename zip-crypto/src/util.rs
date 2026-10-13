//! CSPRNG и constant-time утилиты.

use subtle::ConstantTimeEq;

/// Случайные байты через OS CSPRNG (`getrandom` — без OpenSSL).
pub fn случайные_байты(n: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).map_err(|e| format!("getrandom: {e}"))?;
    Ok(buf)
}

/// Constant-time сравнение (защита от timing-атак).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    bool::from(a.ct_eq(b))
}

/// Constant-time сравнение первых `n` байт.
pub fn constant_time_eq_n(a: &[u8], b: &[u8], n: usize) -> bool {
    if a.len() < n || b.len() < n {
        return false;
    }
    bool::from(a[..n].ct_eq(&b[..n]))
}

#[cfg(test)]
mod тесты {
    use super::*;

    #[test]
    fn random_длина() {
        let b = случайные_байты(32).unwrap();
        assert_eq!(b.len(), 32);
        // Не все нули с высокой вероятностью
        assert!(b.iter().any(|&x| x != 0));
    }

    #[test]
    fn ct_eq() {
        assert!(constant_time_eq(b"abcdef", b"abcdef"));
        assert!(!constant_time_eq(b"abcdef", b"abcdeg"));
        assert!(constant_time_eq_n(b"abc", b"abcdef", 3));
    }
}
