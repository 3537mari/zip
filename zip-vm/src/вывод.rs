//! Захват и маршрутизация вывода VM (native stdout / WASM console / буфер).

use std::cell::RefCell;
use std::sync::Mutex;

thread_local! {
    static БУФЕР: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

/// Глобальный буфер на случай вызова вне thread_local (WASM обычно однопоточен).
static ГЛОБАЛЬНЫЙ: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// Начать накопление вывода (для тестов и WASM API).
pub fn начать_захват() {
    БУФЕР.with(|b| *b.borrow_mut() = Some(Vec::new()));
    if let Ok(mut g) = ГЛОБАЛЬНЫЙ.lock() {
        *g = Some(Vec::new());
    }
}

/// Закончить захват и вернуть накопленный текст.
pub fn закончить_захват() -> String {
    let из_tls = БУФЕР.with(|b| b.borrow_mut().take().unwrap_or_default());
    let из_глоб = ГЛОБАЛЬНЫЙ
        .lock()
        .ok()
        .and_then(|mut g| g.take())
        .unwrap_or_default();
    let bytes = if из_tls.len() >= из_глоб.len() {
        из_tls
    } else {
        из_глоб
    };
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Писать байты в активный буфер или в stdout/console.
pub fn писать(данные: &[u8]) {
    let mut записано = false;
    БУФЕР.with(|b| {
        if let Some(buf) = b.borrow_mut().as_mut() {
            buf.extend_from_slice(данные);
            записано = true;
        }
    });
    if !записано {
        if let Ok(mut g) = ГЛОБАЛЬНЫЙ.lock() {
            if let Some(buf) = g.as_mut() {
                buf.extend_from_slice(данные);
                записано = true;
            }
        }
    }
    if записано {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(s) = std::str::from_utf8(данные) {
            let t = s.trim_end_matches('\n');
            if !t.is_empty() {
                web_sys::console::log_1(&t.into());
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(данные);
        let _ = out.flush();
    }
}

/// Писать в stderr / console.error.
pub fn писать_ошибку(данные: &[u8]) {
    let mut записано = false;
    БУФЕР.with(|b| {
        if let Some(buf) = b.borrow_mut().as_mut() {
            buf.extend_from_slice(данные);
            записано = true;
        }
    });
    if записано {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(s) = std::str::from_utf8(данные) {
            web_sys::console::error_1(&s.into());
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        let mut err = std::io::stderr().lock();
        let _ = err.write_all(данные);
        let _ = err.flush();
    }
}
