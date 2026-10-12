//! Fixed-point: C-bootstrap и selfc (ZIP) дают одинаковый ZIPB для главная.zip.

use std::path::PathBuf;
use std::process::Command;

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn zipc() -> PathBuf {
    let r = корень();
    let exe = r.join("bootstrap/zipc.exe");
    if exe.exists() {
        return exe;
    }
    let unix = r.join("bootstrap/zipc");
    if unix.exists() {
        return unix;
    }
    let _ = Command::new("pwsh")
        .args(["-File", "bootstrap/build.ps1"])
        .current_dir(&r)
        .status();
    if exe.exists() {
        exe
    } else {
        unix
    }
}

fn vm() -> PathBuf {
    let r = корень();
    let release = r.join("zip-vm/target/release/zip-vm.exe");
    if release.exists() {
        return release;
    }
    let _ = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--manifest-path",
            r.join("zip-vm/Cargo.toml").to_str().unwrap(),
        ])
        .current_dir(&r)
        .env("CARGO_TARGET_DIR", r.join("zip-vm/target"))
        .status();
    if release.exists() {
        release
    } else {
        r.join("zip-vm/target/release/zip-vm")
    }
}

#[test]
fn тест_fixed_point_selfc() {
    let root = корень();
    let src = root.join("zip-self-zip/главное.zip");
    assert!(src.exists(), "нет {}", src.display());

    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let v1 = std::env::temp_dir().join(format!("selfc_v1_{id}.zipb"));
    let v2 = std::env::temp_dir().join(format!("selfc_v2_{id}.zipb"));
    let v3 = std::env::temp_dir().join(format!("selfc_v3_{id}.zipb"));

    let c1 = Command::new(zipc())
        .args([src.to_str().unwrap(), v1.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .expect("zipc");
    assert!(
        c1.status.success(),
        "stage0: {}",
        String::from_utf8_lossy(&c1.stderr)
    );

    let vm_bin = vm();
    let c2 = Command::new(&vm_bin)
        .args([
            v1.to_str().unwrap(),
            src.to_str().unwrap(),
            v2.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("vm1");
    assert!(
        c2.status.success(),
        "stage1: {}\n{}",
        String::from_utf8_lossy(&c2.stderr),
        String::from_utf8_lossy(&c2.stdout)
    );

    let c3 = Command::new(&vm_bin)
        .args([
            v2.to_str().unwrap(),
            src.to_str().unwrap(),
            v3.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("vm2");
    assert!(
        c3.status.success(),
        "stage2: {}",
        String::from_utf8_lossy(&c3.stderr)
    );

    let b1 = std::fs::read(&v1).unwrap();
    let b2 = std::fs::read(&v2).unwrap();
    let b3 = std::fs::read(&v3).unwrap();
    assert_eq!(&b1[0..4], b"ZIPB");
    assert_eq!(b1, b2, "v1 != v2 (fixed-point stage1)");
    assert_eq!(b2, b3, "v2 != v3 (fixed-point stage2)");
}
