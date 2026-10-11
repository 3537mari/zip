# Installers для ZIP

Готовые пакеты и скрипты сборки для всех платформ.

| Платформа | Формат | Скрипт |
|-----------|--------|--------|
| Windows | .msi | `windows/build.ps1` + `zip.wxs` |
| Debian/Ubuntu | .deb | `linux/build-deb.sh` |
| Red Hat/Fedora | .rpm | `linux/build-rpm.sh` / `zip.spec` |
| macOS | .dmg | `macos/build-dmg.sh` |
| Universal | .sh | `offline/package.sh` |

Сборка бинарников: `cargo build --release` или `bootstrap/` (C).

CI: [`.github/workflows/build-installers.yml`](../.github/workflows/build-installers.yml)

Скачивание готовых артефактов — [GitHub Releases](https://github.com/3537mari/zip/releases).
