//! Тесты AES-256-GCM на чистом ZIP (NIST + round-trip + подделка тега).

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn aes_source() -> String {
    let p = корень().join("zip-std").join("крипто_aes_zip.zip");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn zipc() -> PathBuf {
    let root = корень();
    let cand = root.join("bootstrap").join("zipc.exe");
    if cand.exists() {
        return cand;
    }
    root.join("bootstrap").join("zipc")
}

fn zip_vm() -> PathBuf {
    let root = корень();
    for rel in [
        "zip-vm/target/release/zip-vm.exe",
        "zip-vm/target/release/zip-vm",
        "zip-vm/target/debug/zip-vm.exe",
        "zip-vm/target/debug/zip-vm",
    ] {
        let p = root.join(rel);
        if p.exists() {
            return p;
        }
    }
    root.join("zip-vm/target/release/zip-vm")
}

fn запустить_zip(программа: &str) -> String {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir();
    let файл = dir.join(format!("тест_aes_{id}.zip"));
    let zipb = dir.join(format!("тест_aes_{id}.zipb"));
    std::fs::write(&файл, программа).unwrap();

    let st = Command::new(zipc())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .status()
        .expect("zipc");
    assert!(st.success(), "zipc failed");

    let out = Command::new(zip_vm())
        .arg(zipb.to_str().unwrap())
        .output()
        .expect("zip-vm");
    assert!(
        out.status.success(),
        "zip-vm failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// NIST AES-256-GCM: key=0, iv=0, pt=16 zero bytes.
#[test]
fn тест_nist_aes_vector_1() {
    let программа = format!(
        r#"{}
функция главная() {{
    ключ = [0; 32]
    iv = [0; 12]
    plaintext = [0; 16]
    шифротекст = [0; 16]
    тег = [0; 16]
    aes256_gcm_шифровать_с_iv(plaintext, 16, ключ, iv, шифротекст, тег)
    для и в 0..16 {{
        напечатать_число(шифротекст[и])
        напечатать_строку(" ")
    }}
    напечатать_строку("|")
    для и в 0..16 {{
        напечатать_число(тег[и])
        напечатать_строку(" ")
    }}
    возврат(0)
}}
"#,
        aes_source()
    );
    let вывод = запустить_zip(&программа);
    let norm = вывод.split_whitespace().collect::<Vec<_>>().join(" ");
    let ct = "206 167 64 61 77 96 107 110 7 78 197 211 186 243 157 24";
    let tag = "208 209 200 167 153 153 107 240 38 91 152 181 212 138 185 25";
    assert!(norm.contains(ct), "NIST ciphertext mismatch: {вывод}");
    assert!(norm.contains(tag), "NIST tag mismatch: {вывод}");
}

/// Round-trip: encrypt → decrypt = original.
#[test]
fn тест_aes_round_trip_zip() {
    let программа = format!(
        r#"{}
функция главная() {{
    ключ = [1; 32]
    для и в 0..32 {{
        ключ[и] = и + 1
    }}
    iv = [0; 12]
    для и в 0..12 {{
        iv[и] = и + 10
    }}
    данные = [0; 18]
    для и в 0..18 {{
        данные[и] = и + 65
    }}
    шифротекст = [0; 18]
    тег = [0; 16]
    открытый = [0; 18]
    aes256_gcm_шифровать_с_iv(данные, 18, ключ, iv, шифротекст, тег)
    код = aes256_gcm_расшифровать(шифротекст, 18, ключ, iv, тег, открытый)
    если (код != 0) {{
        напечатать_строку("FAIL_AUTH")
        возврат(1)
    }}
    ок = 1
    для и в 0..18 {{
        если (открытый[и] != данные[и]) {{
            ок = 0
        }}
    }}
    если (ок == 1) {{
        напечатать_строку("ROUND_TRIP_OK")
    }} иначе {{
        напечатать_строку("ROUND_TRIP_BAD")
    }}
    возврат(0)
}}
"#,
        aes_source()
    );
    let вывод = запустить_zip(&программа);
    assert!(
        вывод.contains("ROUND_TRIP_OK"),
        "Round-trip failed: {вывод}"
    );
}

/// Подделка тега → ошибка.
#[test]
fn тест_aes_подделка_тега() {
    let программа = format!(
        r#"{}
функция главная() {{
    ключ = [7; 32]
    iv = [3; 12]
    данные = [0; 8]
    для и в 0..8 {{
        данные[и] = и + 1
    }}
    шифротекст = [0; 8]
    тег = [0; 16]
    открытый = [0; 8]
    aes256_gcm_шифровать_с_iv(данные, 8, ключ, iv, шифротекст, тег)
    тег[0] = байт_xor(тег[0], 1)
    код = aes256_gcm_расшифровать(шифротекст, 8, ключ, iv, тег, открытый)
    если (код != 0) {{
        напечатать_строку("AUTH_FAIL")
    }} иначе {{
        напечатать_строку("AUTH_BAD_ACCEPT")
    }}
    возврат(0)
}}
"#,
        aes_source()
    );
    let вывод = запустить_zip(&программа);
    assert!(
        вывод.contains("AUTH_FAIL"),
        "Tag forgery not detected: {вывод}"
    );
}
