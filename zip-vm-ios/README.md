# ZIP-VM для iOS (C-API + Swift)

Static library `libzip_vm_ios.a` → `ZipVM.xcframework` + Swift-обёртка `Zip`.

## Сборка XCFramework (macOS + Xcode)

```bash
bash scripts/build-xcframework.sh
```

Targets: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`.

## Swift

```swift
let result = Zip.compileAndRun("""
функция главная() {
    напечатать_строку("Привет из iOS!")
    возврат(42)
}
""")
print(result ?? "Ошибка")
```

## Демо

```bash
# после build-xcframework.sh
cd demo
xcodebuild -scheme ZipDemo -sdk iphonesimulator \
  -destination 'generic/platform=iOS Simulator' build
```

Подробнее: [docs/IOS.md](../docs/IOS.md).
