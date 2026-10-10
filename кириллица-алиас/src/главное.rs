//! CLI: создать кириллические алиасы рядом с ASCII-бинарниками.

use std::env;
use std::path::PathBuf;

fn main() {
    let каталог = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release"));

    println!("Алиасы в {}", каталог.display());
    for (имя, рез) in cyrillic_alias::создать_все_в(&каталог) {
        match рез {
            Ok(p) => println!("  ✓ {имя} → {}", p.display()),
            Err(е) => println!("  · {имя}: {е}"),
        }
    }
}
