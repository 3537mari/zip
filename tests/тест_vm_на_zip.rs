use std::process::Command;

#[test]
fn тест_vm_компилируется() {
    let программа = r#"
функция главная() {
    возврат(42)
}
"#;
    let файл = std::env::temp_dir().join("тест_vm.zip");
    std::fs::write(&файл, &программа).unwrap();
    let zipb = std::env::temp_dir().join("тест_vm.zipb");
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let zipc = if root.join("bootstrap/zipc.exe").exists() {
        root.join("bootstrap/zipc.exe")
    } else {
        root.join("bootstrap/zipc")
    };
    assert!(Command::new(zipc)
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    let zipb_bytes = std::fs::read(&zipb).unwrap();
    assert_eq!(&zipb_bytes[0..4], b"ZIPB");
    let версия = u16::from_le_bytes([zipb_bytes[4], zipb_bytes[5]]);
    assert_eq!(версия, 3);
}
