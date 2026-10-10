use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn zipm_bin() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push("debug");
    if cfg!(windows) {
        // prefer ASCII name
        let a = p.join("zipm.exe");
        if a.exists() {
            return a;
        }
        p.join("зипм.exe")
    } else {
        let a = p.join("zipm");
        if a.exists() {
            return a;
        }
        p.join("зипм")
    }
}

fn release_zipm() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target/release");
    let a = if cfg!(windows) {
        p.join("zipm.exe")
    } else {
        p.join("zipm")
    };
    if a.exists() {
        a
    } else {
        zipm_bin()
    }
}

#[test]
fn создание_пакета() {
    let tmp = tempfile::tempdir().unwrap();
    let имя = tmp.path().join("тестовый_пакет");
    let статус = Command::new(release_zipm())
        .args(["create", имя.file_name().unwrap().to_str().unwrap()])
        .current_dir(tmp.path())
        .status()
        .expect("запуск зипм");
    assert!(статус.success());
    assert!(имя.join("зипм.манифест").exists() || имя.join("zipm.toml").exists());
    assert!(имя.join("src/главная.zip").exists());
}

#[test]
fn добавление_зависимости() {
    let tmp = tempfile::tempdir().unwrap();
    let имя = "pkg_deps";
    assert!(Command::new(release_zipm())
        .args(["create", имя])
        .current_dir(tmp.path())
        .status()
        .unwrap()
        .success());
    let pkg = tmp.path().join(имя);
    assert!(Command::new(release_zipm())
        .args(["add", "тензоры", "1.0.0"])
        .current_dir(&pkg)
        .status()
        .unwrap()
        .success());
    let м = fs::read_to_string(pkg.join("zipm.toml")).unwrap();
    assert!(м.contains("тензоры") || fs::read_to_string(pkg.join("зипм.манифест")).unwrap().contains("тензоры"));
}

#[test]
fn сборка_и_запуск() {
    // Нужен собранный zipc
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let zipc = root.join("zipc/target/release/zipc.exe");
    let zipc_unix = root.join("zipc/target/release/zipc");
    if !zipc.exists() && !zipc_unix.exists() {
        eprintln!("skip: zipc не собран");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let имя = "pkg_run";
    assert!(Command::new(release_zipm())
        .args(["create", имя])
        .current_dir(tmp.path())
        .env("ZIP_ROOT", &root)
        .status()
        .unwrap()
        .success());
    let pkg = tmp.path().join(имя);
    let build = Command::new(release_zipm())
        .args(["build", "--release"])
        .current_dir(&pkg)
        .env("ZIP_ROOT", &root)
        .status()
        .unwrap();
    assert!(build.success(), "сборка должна пройти");

    let out = Command::new(release_zipm())
        .arg("run")
        .current_dir(&pkg)
        .env("ZIP_ROOT", &root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        combined.contains("Привет из ZIP-пакета") || out.status.success(),
        "unexpected output: {combined}"
    );
}
