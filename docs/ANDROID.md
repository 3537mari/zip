# ZIP на Android (NDK + JNI)

## Что нужно знать

После v7.0.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**.
Для Android пересобирается только Rust-VM как **JNI shared library**
(`libzip_vm_android.so`), вызываемая из Kotlin/Java.

ZIPB тот же: `03ec79b9…c6a98c59`.

## Сборка `.so`

```bash
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk

# NDK (Android Studio SDK Manager или sdkmanager)
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"   # пример

cd zip-vm-android
cargo ndk -t arm64-v8a -t x86_64 \
    -o ./app/src/main/jniLibs \
    build --release
```

Артефакты:

```
app/src/main/jniLibs/
├── arm64-v8a/libzip_vm_android.so
└── x86_64/libzip_vm_android.so
```

## Kotlin API

```kotlin
import org.ziplang.zip.Zip

val результат = Zip.скомпилироватьИЗапустить("""
функция главная() {
    напечатать_строку("Привет из Android!")
    возврат(42)
}
""")
```

| Kotlin | JNI (ASCII) | Назначение |
|--------|-------------|------------|
| `Zip.скомпилировать` | `compile` | ZIP → ZIPB hex |
| `Zip.запустить` | `run` | ZIPB → вывод |
| `Zip.скомпилироватьИЗапустить` | `compileAndRun` | ZIP → вывод |
| `Zip.версия` | `version` | версия crate |

## APK / демо

```bash
cd zip-vm-android
gradle assembleDebug   # или ./gradlew после wrapper
```

Требования: **Android 5.0+ (API 21)**, ABI `arm64-v8a` / `x86_64`.

## CI

См. [`.github/workflows/android.yml`](../.github/workflows/android.yml):
сборка `.so`, APK, instrumentation-тесты на x86_64-эмуляторе.

## Что дальше

- iOS (Swift + C-API) — ✅ [docs/IOS.md](IOS.md) (v7.2.0)
