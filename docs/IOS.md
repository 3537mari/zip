# ZIP на iOS (Swift + Objective-C bridge)

## Что нужно знать

После v7.0.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**:
- Лексер, парсер, генератор ZIPB — на **чистом ZIP**
- Fixed-point: v2 = v3 = v4 = `03ec79b9...c6a98c59`
- ZIP-байткод **один и тот же** на всех платформах

**Для iOS нужно только пересобрать Rust-VM** как static library.

## Особенности iOS

- **Static library** (`.a`), не `.so` (App Store не принимает произвольные dylib).
- **XCFramework** — универсальный бандл для устройства + симулятора.
- **C-API** + Swift-обёртка (не JNI, как на Android).
- Bridging header / `module.modulemap` для Objective-C / Swift.

## Сборка

```bash
# 1. Targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# 2–4. Скрипт (cargo + lipo + xcodebuild -create-xcframework)
cd zip-vm-ios
bash scripts/build-xcframework.sh
```

Артефакт: `zip-vm-ios/ZipVM.xcframework/`

| Slice | Архитектуры |
|-------|-------------|
| `ios-arm64` | устройство |
| `ios-arm64_x86_64-simulator` | симулятор (lipo) |

## C-API

| Функция | Назначение |
|---------|------------|
| `zip_compile` | ZIP → ZIPB hex |
| `zip_run` | ZIPB hex → вывод |
| `zip_compile_and_run` | ZIP → вывод |
| `zip_free` | освободить строку |
| `zip_version` | версия crate |
| `zip_last_success` | успех последней операции |

Заголовок: [`zip-vm-ios/include/zip.h`](../zip-vm-ios/include/zip.h).

## Использование из Swift

```swift
let result = Zip.compileAndRun("""
функция главная() {
    напечатать_строку("Привет из iOS!")
    возврат(42)
}
""")
print(result ?? "Ошибка")
```

| Swift | C-API |
|-------|-------|
| `Zip.compile` | `zip_compile` |
| `Zip.run` | `zip_run` |
| `Zip.compileAndRun` | `zip_compile_and_run` |
| `Zip.version` | `zip_version` |

В Xcode: добавить `ZipVM.xcframework` в Frameworks (Do Not Embed для static),
указать Bridging Header:

```objc
#import "zip.h"
```

## Демо

```bash
cd zip-vm-ios/demo
xcodebuild -scheme ZipDemo \
  -sdk iphonesimulator \
  -destination 'generic/platform=iOS Simulator' \
  CODE_SIGNING_ALLOWED=NO \
  build
```

## Требования

- iOS 13.0+
- Xcode 15+
- Swift 5.9+
- macOS для сборки targets / XCFramework

## CI

См. [`.github/workflows/ios.yml`](../.github/workflows/ios.yml):
static libs → lipo → XCFramework → сборка/тесты демо на симуляторе.

## Что дальше

- Криптография: см. [КРИПТОГРАФИЯ.md](КРИПТОГРАФИЯ.md) (если есть)
- GPU / Metal — последующие релизы v7.x
