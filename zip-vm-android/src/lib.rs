//! JNI-обёртка ZIP-VM для Android.
//!
//! JNI-имена — ASCII (`compile` / `run` / …). Kotlin [`Zip`] даёт кириллический API.

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;

/// `org.ziplang.zip.Zip.compile(String): String`
#[no_mangle]
pub extern "system" fn Java_org_ziplang_zip_Zip_compile(
    mut env: JNIEnv,
    _class: JClass,
    source: JString,
) -> jstring {
    let исходник: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => return ошибка_jstring(&mut env, &format!("Ошибка JNI: {e}")),
    };

    match zip_vm::zip_компилятор::скомпилировать(&исходник) {
        Ok(байткод) => {
            let hex = hex_encode(&байткод);
            env.new_string(&hex)
                .unwrap_or_else(|_| env.new_string("Ошибка JNI: new_string").unwrap())
                .into_raw()
        }
        Err(e) => ошибка_jstring(&mut env, &format!("Ошибка компиляции: {e}")),
    }
}

/// `org.ziplang.zip.Zip.run(String): String`
#[no_mangle]
pub extern "system" fn Java_org_ziplang_zip_Zip_run(
    mut env: JNIEnv,
    _class: JClass,
    bytecode_hex: JString,
) -> jstring {
    let hex: String = match env.get_string(&bytecode_hex) {
        Ok(s) => s.into(),
        Err(e) => return ошибка_jstring(&mut env, &format!("Ошибка JNI: {e}")),
    };

    let байткод = match hex_decode(&hex) {
        Ok(b) => b,
        Err(e) => return ошибка_jstring(&mut env, &format!("Ошибка hex: {e}")),
    };

    match zip_vm::zip_компилятор::запустить(&байткод) {
        Ok(вывод) => env
            .new_string(&вывод)
            .unwrap_or_else(|_| env.new_string("Ошибка JNI: new_string").unwrap())
            .into_raw(),
        Err(e) => ошибка_jstring(&mut env, &format!("Ошибка VM: {e}")),
    }
}

/// `org.ziplang.zip.Zip.compileAndRun(String): String`
#[no_mangle]
pub extern "system" fn Java_org_ziplang_zip_Zip_compileAndRun(
    mut env: JNIEnv,
    _class: JClass,
    source: JString,
) -> jstring {
    let исходник: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => return ошибка_jstring(&mut env, &format!("Ошибка JNI: {e}")),
    };

    match zip_vm::zip_компилятор::скомпилировать_и_запустить(&исходник) {
        Ok(вывод) => env
            .new_string(&вывод)
            .unwrap_or_else(|_| env.new_string("Ошибка JNI: new_string").unwrap())
            .into_raw(),
        Err(e) => ошибка_jstring(&mut env, &format!("Ошибка: {e}")),
    }
}

/// `org.ziplang.zip.Zip.version(): String`
#[no_mangle]
pub extern "system" fn Java_org_ziplang_zip_Zip_version(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    env.new_string(env!("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| env.new_string("?").unwrap())
        .into_raw()
}

fn hex_encode(данные: &[u8]) -> String {
    let mut s = String::with_capacity(данные.len() * 2);
    for b in данные {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("нечётная длина hex".into());
    }
    let mut результат = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let a = bytes[i] as char;
        let b = bytes[i + 1] as char;
        let байт =
            u8::from_str_radix(&format!("{a}{b}"), 16).map_err(|e| format!("hex: {e}"))?;
        результат.push(байт);
        i += 2;
    }
    Ok(результат)
}

fn ошибка_jstring(env: &mut JNIEnv, сообщение: &str) -> jstring {
    env.new_string(сообщение)
        .unwrap_or_else(|_| env.new_string("JNI error").unwrap())
        .into_raw()
}
