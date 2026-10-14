//! Встроенные STD v4 (опорная реализация для «чистого ZIP» STD).

use crate::встроенные_v3::Встроенные;
use crate::исполнитель_v3::ЗначениеV3;
use std::collections::HashMap;
use std::sync::Mutex;

static SIPRA: Mutex<Option<HashMap<i64, Vec<i64>>>> = Mutex::new(None);
static TENSORS: Mutex<Option<HashMap<i64, Vec<f64>>>> = Mutex::new(None);
static LAYERS: Mutex<Option<HashMap<i64, (i64, i64)>>> = Mutex::new(None);
static NEXT_ID: Mutex<i64> = Mutex::new(1);
static RAND: Mutex<u64> = Mutex::new(12345);

fn next_id() -> i64 {
    let mut g = NEXT_ID.lock().unwrap();
    let id = *g;
    *g += 1;
    id
}

fn как_строка(з: &ЗначениеV3) -> String {
    match з {
        ЗначениеV3::Строка(s) => s.clone(),
        другой => другой.to_string(),
    }
}

fn i64_arg(арг: &[ЗначениеV3], i: usize) -> i64 {
    арг.get(i).map(|z| z.как_целое()).unwrap_or(0)
}

pub fn зарегистрировать(ф: &mut Встроенные) {
    ф.регистрация_внешняя("zip_std_strlen", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(s.chars().count() as i64))
    });
    ф.регистрация_внешняя("zip_std_strcat", |арг| {
        let a = арг.first().map(как_строка).unwrap_or_default();
        let b = арг.get(1).map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Строка(format!("{a}{b}")))
    });
    ф.регистрация_внешняя("zip_std_strdup", |арг| {
        Ok(ЗначениеV3::Строка(
            арг.first().map(как_строка).unwrap_or_default(),
        ))
    });
    ф.регистрация_внешняя("zip_std_strcmp", |арг| {
        let a = арг.first().map(как_строка).unwrap_or_default();
        let b = арг.get(1).map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(match a.cmp(&b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }))
    });

    ф.регистрация_внешняя("zip_std_sipra_new", |арг| {
        let cap = i64_arg(арг, 0).max(4) as usize;
        let id = next_id();
        let mut g = SIPRA.lock().unwrap();
        if g.is_none() {
            *g = Some(HashMap::new());
        }
        g.as_mut().unwrap().insert(id, Vec::with_capacity(cap));
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_std_sipra_push", |арг| {
        let id = i64_arg(арг, 0);
        let v = i64_arg(арг, 1);
        let mut g = SIPRA.lock().unwrap();
        if let Some(m) = g.as_mut() {
            if let Some(vec) = m.get_mut(&id) {
                vec.push(v);
            }
        }
        Ok(ЗначениеV3::Целое(v))
    });
    ф.регистрация_внешняя("zip_std_sipra_get", |арг| {
        let id = i64_arg(арг, 0);
        let idx = i64_arg(арг, 1) as usize;
        let g = SIPRA.lock().unwrap();
        let v = g
            .as_ref()
            .and_then(|m| m.get(&id))
            .and_then(|v| v.get(idx))
            .copied()
            .unwrap_or(0);
        Ok(ЗначениеV3::Целое(v))
    });
    ф.регистрация_внешняя("zip_std_sipra_len", |арг| {
        let id = i64_arg(арг, 0);
        let g = SIPRA.lock().unwrap();
        let n = g
            .as_ref()
            .and_then(|m| m.get(&id))
            .map(|v| v.len() as i64)
            .unwrap_or(0);
        Ok(ЗначениеV3::Целое(n))
    });

    ф.регистрация_внешняя("zip_std_tensor_new", |арг| {
        let r0 = i64_arg(арг, 0).max(1);
        let r1 = i64_arg(арг, 1).max(1);
        let r2 = i64_arg(арг, 2).max(1);
        let r3 = i64_arg(арг, 3).max(1);
        let total = (r0 * r1 * r2 * r3).max(r0);
        let id = next_id();
        let mut g = TENSORS.lock().unwrap();
        if g.is_none() {
            *g = Some(HashMap::new());
        }
        g.as_mut()
            .unwrap()
            .insert(id, vec![0.0; total as usize]);
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_std_tensor_fill", |арг| {
        let id = i64_arg(арг, 0);
        let val = арг.get(1).map(|z| z.как_вещественное()).unwrap_or(0.0);
        let mut g = TENSORS.lock().unwrap();
        if let Some(m) = g.as_mut() {
            if let Some(t) = m.get_mut(&id) {
                for x in t.iter_mut() {
                    *x = val;
                }
            }
        }
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_std_tensor_total", |арг| {
        let id = i64_arg(арг, 0);
        let g = TENSORS.lock().unwrap();
        let n = g
            .as_ref()
            .and_then(|m| m.get(&id))
            .map(|v| v.len() as i64)
            .unwrap_or(0);
        Ok(ЗначениеV3::Целое(n))
    });
    ф.регистрация_внешняя("zip_std_gemm", |арг| {
        let m = i64_arg(арг, 2).max(1);
        let n = i64_arg(арг, 4).max(1);
        let id = next_id();
        let mut g = TENSORS.lock().unwrap();
        if g.is_none() {
            *g = Some(HashMap::new());
        }
        g.as_mut()
            .unwrap()
            .insert(id, vec![0.0; (m * n) as usize]);
        Ok(ЗначениеV3::Целое(id))
    });

    ф.регистрация_внешняя("zip_std_layer_new", |арг| {
        let inp = i64_arg(арг, 0);
        let out = i64_arg(арг, 1);
        let id = next_id();
        let mut g = LAYERS.lock().unwrap();
        if g.is_none() {
            *g = Some(HashMap::new());
        }
        g.as_mut().unwrap().insert(id, (inp, out));
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_std_layer_forward", |арг| {
        let layer = i64_arg(арг, 0);
        let g = LAYERS.lock().unwrap();
        let out_dim = g
            .as_ref()
            .and_then(|m| m.get(&layer))
            .map(|(_, o)| *o)
            .unwrap_or(1);
        let id = next_id();
        let mut tg = TENSORS.lock().unwrap();
        if tg.is_none() {
            *tg = Some(HashMap::new());
        }
        tg.as_mut()
            .unwrap()
            .insert(id, vec![0.5; out_dim.max(1) as usize]);
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_std_layer_backward", |арг| {
        Ok(ЗначениеV3::Целое(i64_arg(арг, 1)))
    });
    ф.регистрация_внешняя("zip_std_rand", |арг| {
        let m = i64_arg(арг, 0).max(1) as u64;
        let mut s = RAND.lock().unwrap();
        *s = s.wrapping_mul(1103515245).wrapping_add(12345);
        Ok(ЗначениеV3::Целое((*s % m) as i64))
    });
    ф.регистрация_внешняя("zip_std_sha256", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        let len = i64_arg(арг, 1);
        let bytes = if len > 0 && (len as usize) <= s.len() {
            s.as_bytes()[..len as usize].to_vec()
        } else {
            s.into_bytes()
        };
        Ok(ЗначениеV3::Строка(zip_crypto::sha::ша256_hex(&bytes)))
    });
    ф.регистрация_внешняя("zip_std_sha3_256", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Строка(hex::encode(zip_crypto::ша3_256(
            s.as_bytes(),
        ))))
    });
    ф.регистрация_внешняя("zip_crypto_random", |арг| {
        let n = i64_arg(арг, 0).max(0) as usize;
        let b = zip_crypto::случайные_байты(n)?;
        Ok(ЗначениеV3::Строка(hex::encode(b)))
    });
    ф.регистрация_внешняя("zip_crypto_kyber_keygen", |_арг| {
        let (pk, sk) = zip_crypto::kyber1024_генерация_ключей()?;
        Ok(ЗначениеV3::Строка(format!(
            "{}|{}",
            hex::encode(pk),
            hex::encode(sk)
        )))
    });
    ф.регистрация_внешняя("zip_crypto_kyber_encap", |арг| {
        let pk_hex = арг.first().map(как_строка).unwrap_or_default();
        let pk = hex::decode(pk_hex.trim()).map_err(|e| format!("pk: {e}"))?;
        let (ct, ss) = zip_crypto::kyber1024_инкапсуляция(&pk)?;
        Ok(ЗначениеV3::Строка(format!(
            "{}|{}",
            hex::encode(ct),
            hex::encode(ss)
        )))
    });
    ф.регистрация_внешняя("zip_crypto_kyber_decap", |арг| {
        let ct_hex = арг.first().map(как_строка).unwrap_or_default();
        let sk_hex = арг.get(1).map(как_строка).unwrap_or_default();
        let ct = hex::decode(ct_hex.trim()).map_err(|e| format!("ct: {e}"))?;
        let sk = hex::decode(sk_hex.trim()).map_err(|e| format!("sk: {e}"))?;
        let ss = zip_crypto::kyber1024_декапсуляция(&ct, &sk)?;
        Ok(ЗначениеV3::Строка(hex::encode(ss)))
    });
    ф.регистрация_внешняя("zip_crypto_dilithium_keygen", |_арг| {
        let (pk, sk) = zip_crypto::dilithium5_генерация_ключей()?;
        Ok(ЗначениеV3::Строка(format!(
            "{}|{}",
            hex::encode(pk),
            hex::encode(sk)
        )))
    });
    ф.регистрация_внешняя("zip_crypto_dilithium_sign", |арг| {
        let msg = арг.first().map(как_строка).unwrap_or_default();
        let sk_hex = арг.get(1).map(как_строка).unwrap_or_default();
        let sk = hex::decode(sk_hex.trim()).map_err(|e| format!("sk: {e}"))?;
        let sig = zip_crypto::dilithium5_подписать(msg.as_bytes(), &sk)?;
        Ok(ЗначениеV3::Строка(hex::encode(sig)))
    });
    ф.регистрация_внешняя("zip_crypto_dilithium_verify", |арг| {
        let msg = арг.first().map(как_строка).unwrap_or_default();
        let sig_hex = арг.get(1).map(как_строка).unwrap_or_default();
        let pk_hex = арг.get(2).map(как_строка).unwrap_or_default();
        let sig = hex::decode(sig_hex.trim()).map_err(|e| format!("sig: {e}"))?;
        let pk = hex::decode(pk_hex.trim()).map_err(|e| format!("pk: {e}"))?;
        let ok = zip_crypto::dilithium5_проверить(msg.as_bytes(), &sig, &pk);
        Ok(ЗначениеV3::Целое(if ok { 1 } else { 0 }))
    });
    ф.регистрация_внешняя("zip_crypto_ct_eq", |арг| {
        let a = арг.first().map(как_строка).unwrap_or_default();
        let b = арг.get(1).map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(if zip_crypto::constant_time_eq(
            a.as_bytes(),
            b.as_bytes(),
        ) {
            1
        } else {
            0
        }))
    });
    ф.регистрация_внешняя("zip_std_json_parse", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(s.len() as i64))
    });
    ф.регистрация_внешняя("zip_std_csv_parse", |арг| {
        let s = арг.first().map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(s.lines().count() as i64))
    });
    ф.регистрация_внешняя("zip_std_regex_match", |арг| {
        let pat = арг.first().map(как_строка).unwrap_or_default();
        let text = арг.get(1).map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Целое(if text.contains(&pat) { 1 } else { 0 }))
    });
    ф.регистрация_внешняя("zip_std_http_get", |арг| {
        let url = арг.first().map(как_строка).unwrap_or_default();
        Ok(ЗначениеV3::Строка(format!("GET:{url}")))
    });

    // Алиасы для программ STD на ZIP
    for name in [
        "создать_тензор",
        "заполнить_тензор",
        "тензор_всего",
        "матричное_умножение",
        "создать_слой",
        "прямой_проход_слоя",
        "обратный_проход_слоя",
        "случайное_целое",
        "хеш_ша256",
        "хеш_ша3_256",
        "случайные_байты",
        "kyber1024_генерация_ключей",
        "kyber1024_инкапсуляция",
        "kyber1024_декапсуляция",
        "dilithium5_генерация_ключей",
        "dilithium5_подписать",
        "dilithium5_проверить",
        "constant_time_eq",
        "json_разобрать",
        "csv_разобрать",
        "regex_совпадает",
        "http_get",
    ] {
        let target = match name {
            "создать_тензор" => "zip_std_tensor_new",
            "заполнить_тензор" => "zip_std_tensor_fill",
            "тензор_всего" => "zip_std_tensor_total",
            "матричное_умножение" => "zip_std_gemm",
            "создать_слой" => "zip_std_layer_new",
            "прямой_проход_слоя" => "zip_std_layer_forward",
            "обратный_проход_слоя" => "zip_std_layer_backward",
            "случайное_целое" => "zip_std_rand",
            "хеш_ша256" => "zip_std_sha256",
            "хеш_ша3_256" => "zip_std_sha3_256",
            "случайные_байты" => "zip_crypto_random",
            "kyber1024_генерация_ключей" => "zip_crypto_kyber_keygen",
            "kyber1024_инкапсуляция" => "zip_crypto_kyber_encap",
            "kyber1024_декапсуляция" => "zip_crypto_kyber_decap",
            "dilithium5_генерация_ключей" => "zip_crypto_dilithium_keygen",
            "dilithium5_подписать" => "zip_crypto_dilithium_sign",
            "dilithium5_проверить" => "zip_crypto_dilithium_verify",
            "constant_time_eq" => "zip_crypto_ct_eq",
            "json_разобрать" => "zip_std_json_parse",
            "csv_разобрать" => "zip_std_csv_parse",
            "regex_совпадает" => "zip_std_regex_match",
            "http_get" => "zip_std_http_get",
            _ => continue,
        };
        ф.регистрация_алиас(name, target);
    }
}
