//! Наличие плагинов Neovim/Emacs и парсинг тестового примера.

use std::path::PathBuf;
use zipc::{разобрать_исходник, Узел};

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn тест_структура_nvim_zip() {
    let корень = корень();
    for rel in [
        "nvim-zip/plugin/zip.lua",
        "nvim-zip/lua/zip/init.lua",
        "nvim-zip/lua/zip/lsp.lua",
        "nvim-zip/lua/zip/config.lua",
        "nvim-zip/syntax/zip.vim",
        "nvim-zip/ftdetect/zip.vim",
        "nvim-zip/queries/zip/highlights.scm",
        "nvim-zip/README.md",
    ] {
        assert!(корень.join(rel).is_file(), "нет файла {rel}");
    }
    let init = std::fs::read_to_string(корень.join("nvim-zip/lua/zip/init.lua")).unwrap();
    assert!(init.contains("function M.setup"));
    assert!(init.contains("<leader>zr"));
}

#[test]
fn тест_структура_emacs_zip() {
    let корень = корень();
    for rel in [
        "emacs-zip/zip-mode.el",
        "emacs-zip/zip-lsp.el",
        "emacs-zip/zip-keymap.el",
        "emacs-zip/README.md",
    ] {
        assert!(корень.join(rel).is_file(), "нет файла {rel}");
    }
    let mode = std::fs::read_to_string(корень.join("emacs-zip/zip-mode.el")).unwrap();
    assert!(mode.contains("define-derived-mode zip-mode"));
    assert!(mode.contains("zip-font-lock-keywords"));
    let lsp = std::fs::read_to_string(корень.join("emacs-zip/zip-lsp.el")).unwrap();
    assert!(lsp.contains("zip-lsp-setup"));
    assert!(lsp.contains("зип-lsp"));
}

#[test]
fn тест_плагин_тест_парсится() {
    let s = std::fs::read_to_string(корень().join("examples/плагин_тест.zip")).unwrap();
    assert!(s.starts_with("#!/usr/bin/env zipc"));
    let д = разобрать_исходник(&s).expect("парсинг плагин_тест.zip");
    match д {
        Узел::Программа { объявления } => {
            assert!(!объявления.is_empty());
        }
        _ => panic!("ожидалась программа"),
    }
}

#[test]
fn тест_docs_плагины() {
    let s = std::fs::read_to_string(корень().join("docs/ПЛАГИНЫ.md")).unwrap();
    assert!(s.contains("nvim-zip"));
    assert!(s.contains("emacs-zip"));
    assert!(s.contains("vscode-zip"));
}
